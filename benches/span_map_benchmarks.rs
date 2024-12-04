use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use span_map::bounds::LeftBound;
use span_map::bounds::RightBound;
use span_map::span::Span;
use span_map::SpanMap;
// Replace with your actual crate name

fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("SpanMap::get");

    // Single range benchmark
    {
        let mut map = SpanMap::<i32, i32>::new();
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(1000)),
            10,
        );

        group.bench_function("single_range", |b| {
            b.iter(|| black_box(map.get(&500).count()));
        });
    }

    // Overlapping ranges benchmark
    {
        let mut map = SpanMap::<i32, i32>::new();
        for i in 0..10 {
            map.insert_span(
                Span::new(
                    LeftBound::Included(i * 100),
                    RightBound::Included((i + 5) * 100),
                ),
                i,
            );
        }

        group.bench_function("overlapping_ranges_10_overlapping", |b| {
            b.iter(|| black_box(map.get(&250).count()));
        });
    }

    // Large value set benchmark
    {
        let mut map = SpanMap::<i32, i32>::new();
        let range = Span::new(LeftBound::Included(1), RightBound::Included(1000));
        for i in 0..100 {
            map.insert_span(range.clone(), i);
        }

        group.bench_function("large_value_set_100_overlapping", |b| {
            b.iter(|| black_box(map.get(&500).count()));
        });
    }

    // Many ranges benchmark
    {
        let mut map = SpanMap::<i32, i32>::new();
        for i in 0..1000 {
            map.insert_span(
                Span::new(LeftBound::Included(i * 2), RightBound::Included(i * 2 + 1)),
                i,
            );
        }

        group.bench_function("many_ranges_1000_no_overlapping", |b| {
            b.iter(|| black_box(map.get(&999).count()));
        });
    }

    // String keys benchmark
    {
        let mut map = SpanMap::<String, i32>::new();
        map.insert_span(
            Span::new(
                LeftBound::Included("aaa".to_string()),
                RightBound::Included("zzz".to_string()),
            ),
            10,
        );

        let query = "mmm".to_string();
        group.bench_function("string_keys_1_overlapping", |b| {
            b.iter(|| black_box(map.get(&query).count()));
        });
    }

    // Worst case benchmark
    {
        let mut map = SpanMap::<i32, i32>::new();
        for i in 0..1000 {
            map.insert_span(
                Span::new(LeftBound::Included(i), RightBound::Included(2000 - i)),
                i,
            );
        }

        group.bench_function("worst_case_1000_overlapping", |b| {
            b.iter(|| black_box(map.get(&1000).count()));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_get);
criterion_main!(benches);
