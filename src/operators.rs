/*
    Some useful custom operators and aggregators
    that aren't provided out-of-the-box in Timely,
    particularly for experiments where we just
    want to aggregate the entire stream.
*/

use super::{Pipeline, Stream, Timestamp};
use crate::util::either::Either;

use std::fs::OpenOptions;
use std::hash::Hash;
use std::io::prelude::*;
use timely::dataflow::operators::vec::{Filter, Map, count::Accumulate};
use timely::dataflow::operators::{Concat, Exchange, Inspect, Operator};

/*
    Output port
    TODO: have not looked into why this is needed in the latest timely
    and if it is used correctly below, need to revisit.
*/

const OUT_PORT: usize = 0;

/*
    Window over the entire input stream, producing a single
    output at the end.

    This version is parallel: it preserves the partition on the
    input stream and thus produces one output per worker.

    Possible Improvements:
    - It would be nice if 'emit' was an FnOnce. Right now I'm not sure
      how to accomplish that.
    - It would also be nice if the window does not persist at all after emit
      is called; perhaps this can be accomplished by using Option magic
      to set the state to None at the end.
*/
pub fn window_all_parallel<'scope, D1, D2, D3, I, F, E, T>(
    name: &str,
    in_stream: Stream<'scope, T, Vec<D1>>,
    init: I,
    fold: F,
    emit: E,
) -> Stream<'scope, T, Vec<D3>>
where
    D1: 'static,
    D2: 'static,
    D3: 'static,
    I: FnOnce() -> D2 + 'static,
    F: Fn(&mut D2, &T, Vec<D1>) + 'static,
    E: Fn(&D2) -> D3 + 'static,
    T: Timestamp + Copy,
{
    in_stream.unary_frontier(Pipeline, name, |capability1, _info| {
        let mut agg = init();
        let cap_time = *capability1.time();
        let mut maybe_cap = Some(capability1);

        move |(input, frontier), output| {
            input.for_each(|cap2, data| {
                let mut data_vec = Vec::new();
                std::mem::swap(data, &mut data_vec);
                fold(&mut agg, cap2.time(), data_vec);
                if *cap2.time() > cap_time {
                    maybe_cap = Some(cap2.retain(OUT_PORT));
                }
            });
            // while let Some((capability2, data)) = input.next() {
            // }
            // Check if entire input is done
            if frontier.is_empty()
                && let Some(cap) = maybe_cap.as_ref()
            {
                output.session(&cap).give(emit(&agg));
                maybe_cap = None;
            }
        }
    })
}

/*
    Window over the entire input stream, producing a single
    output at the end.

    This version forwards all inputs to a single worker,
    and produces only one output item (for that worker).
*/
pub fn window_all<'scope, D1, D2, D3, I, F, E, T>(
    name: &str,
    in_stream: Stream<'scope, T, Vec<D1>>,
    init: I,
    fold: F,
    emit: E,
) -> Stream<'scope, T, Vec<D3>>
where
    D1: timely::ExchangeData + 'static, // input data
    D2: 'static,                        // accumulator
    D3: 'static,                        // output data
    I: FnOnce() -> D2 + 'static,
    F: Fn(&mut D2, &T, Vec<D1>) + 'static,
    E: Fn(&D2) -> D3 + 'static,
    T: Timestamp + Copy,
{
    let in_stream_single = in_stream.exchange(|_x| 0);
    window_all_parallel(
        name,
        in_stream_single,
        || (init(), false),
        move |(x, nonempty), time, data| {
            fold(x, time, data);
            *nonempty = true;
        },
        move |(x, nonempty)| {
            if *nonempty { Some(emit(x)) } else { None }
        },
    )
    .filter(|x| x.is_some())
    .map(|x| x.unwrap()) // guaranteed not to panic
}

/*
    Unary operation on a "singleton" stream, i.e.
    one which has only one element.

    Notes:
    - Panics if called on an input stream which receives more than 2 elements.
    - Waits for an input stream to finish before emitting output, so will hang
      on an input stream which isn't closed even if it only ever gets 1 element.
    - Clones the input once. (This shouldn't be necessary, it's just due to some
      difficulties with ownership, probably because window_all isn't quite
      implemented in the best way yet.)
*/
pub fn single_op_unary<'scope, D1, D2, F, T>(
    name: &str,
    in_stream: Stream<'scope, T, Vec<D1>>,
    op: F,
) -> Stream<'scope, T, Vec<D2>>
where
    D1: timely::ExchangeData + Clone + 'static, // input data
    D2: 'static,                                // output data
    F: Fn(D1) -> D2 + 'static,
    T: Timestamp + Copy,
{
    window_all(
        name,
        in_stream,
        || None,
        |seen, _time, data| {
            for d in data {
                assert!(seen.is_none());
                *seen = Some(d);
            }
        },
        move |seen| op(seen.clone().unwrap()),
    )
}

/*
    Binary operation on two "singleton" streams, i.e.
    streams which have only one element each.
*/
pub fn single_op_binary<'scope, D1, D2, D3, F, T>(
    name: &str,
    in_stream1: Stream<'scope, T, Vec<D1>>,
    in_stream2: Stream<'scope, T, Vec<D2>>,
    op: F,
) -> Stream<'scope, T, Vec<D3>>
where
    D1: timely::ExchangeData + Clone + 'static, // input data 1
    D2: timely::ExchangeData + Clone + 'static, // input data 2
    D3: 'static,                                // output data
    F: Fn(D1, D2) -> D3 + 'static,
    T: Timestamp + Copy,
{
    let stream1 = in_stream1.map(Either::Left);
    let stream2 = in_stream2.map(Either::Right);
    let stream = stream1.concat(stream2);

    window_all(
        name,
        stream,
        || (None, None),
        |(seen1, seen2), _time, data| {
            for d in data {
                match d {
                    Either::Left(d1) => {
                        assert!(seen1.is_none());
                        *seen1 = Some(d1);
                    }
                    Either::Right(d2) => {
                        assert!(seen2.is_none());
                        *seen2 = Some(d2);
                    }
                }
            }
        },
        move |(seen1, seen2)| {
            op(seen1.clone().unwrap(), seen2.clone().unwrap())
        },
    )
}

/*
    Save a stream to a file in append mode.

    Requires as input a formatting function for what to print.
    Returns the input stream (unchanged as output).
    Panics if file handling fails.
*/
pub fn save_to_file<'scope, D, F, T>(
    in_stream: Stream<'scope, T, Vec<D>>,
    filename: &str,
    format: F,
) -> Stream<'scope, T, Vec<D>>
where
    D: timely::ExchangeData, // input data
    F: Fn(&D) -> std::string::String + 'static,
    T: Timestamp,
{
    let mut file =
        OpenOptions::new().create(true).append(true).open(filename).unwrap();
    in_stream.inspect(move |d| {
        writeln!(file, "{}", format(d)).unwrap();
    })
}

/*
    Sum the values in each timestamp.
    (Like count, produce a separate value for each worker)
*/
pub trait Sum<'scope, T: Timestamp> {
    fn sum(self) -> Stream<'scope, T, Vec<usize>>;
}
impl<'scope, T: Timestamp + Hash> Sum<'scope, T>
    for Stream<'scope, T, Vec<usize>>
{
    fn sum(self) -> Stream<'scope, T, Vec<usize>> {
        self.accumulate(0, |sum, data| {
            for &x in data.iter() {
                *sum += x;
            }
        })
    }
}

/*
    A simple implementation of join-by-timestamp.
    This is data-parallel (does not re-partition input streams).
    Note: we could potentially use Differential Dataflow for this,
    and there are other existing implementations of join out there,
    but we only need this very simple case which can be done
    in plain Timely with concat, accumulate, and flat_map.

    Note: would be nice to avoid the first instance of .clone().
    The second instance where we clone for the cross product
    is more unavoidable.
*/
#[rustfmt::skip]
pub fn join_by_timestamp<'scope, D1, D2, T>(
    in_stream1: Stream<'scope, T, Vec<D1>>,
    in_stream2: Stream<'scope, T, Vec<D2>>,
) -> Stream<'scope, T, Vec<(D1, D2)>>
where
    D1: timely::ExchangeData + Clone, // input data 1
    D2: timely::ExchangeData + Clone, // input data 2
    T: Timestamp + Hash,
{
    let stream1 = in_stream1.map(Either::Left);
    let stream2 = in_stream2.map(Either::Right);
    let combined = stream1.concat(stream2);

    // Map items at each timestamp to a pair of vectors
    let collected = combined.accumulate(
        (Vec::new(), Vec::new()),
        |(vec1, vec2), items| {
            for item in items.iter() {
                match item {
                    Either::Left(x) => { vec1.push(x.clone()); }
                    Either::Right(x) => { vec2.push(x.clone()); }
                }
            };
        },
    );

    collected.flat_map(|(vec1, vec2)| {
        let mut result = Vec::new();
        for item1 in vec1.iter() {
            for item2 in vec2.iter() {
                result.push((item1.clone(), item2.clone()));
            }
        }
        result
    })
}
