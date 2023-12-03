
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use futures_util::Stream;
use futures_util::StreamExt;
use futures_util::stream::once;
use futures_util::future;
use std::mem;
use std::time::Duration;

pub fn periodic_buffered_window<V>(period: Duration, stream: impl Stream<Item = V>) -> impl Stream<Item = Vec<V>>
where
    V: Sized,
{
    let stream = stream
        .map(BufferEvent::Item)
        .chain(once(future::ready(BufferEvent::Flush)))
        .chain(once(future::ready(BufferEvent::Completion)));
    let emit = IntervalStream::new(interval(period))
        .map(|_| BufferEvent::Flush);
    let merged_stream;
    {
        use tokio_stream::StreamExt;
        merged_stream = stream.merge(emit);
    }
    merged_stream
        .scan(Vec::<V>::new(), |state, item| {
            let v = match item {
                BufferEvent::Item(v) => {
                    state.push(v);
                    Some(None)
                },
                BufferEvent::Flush => {
                    let buffered = mem::replace(state, Vec::with_capacity(state.capacity()));
                    Some(Some(buffered))
                },
                BufferEvent::Completion => {
                    None
                }
            };
            future::ready(v)
        })
        .filter_map(|v| future::ready(v))
}

enum BufferEvent<V> {
    Item(V),
    Flush,
    Completion,
}
