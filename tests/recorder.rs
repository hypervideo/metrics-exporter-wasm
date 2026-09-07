use metrics::{
    Key,
    Recorder,
};
use metrics_exporter_wasm::{
    Event,
    MetricOperation,
    MetricType,
    WasmRecorder,
};
use tokio::sync::broadcast::error::TryRecvError;

#[test]
fn records_owned_keys_and_macro_handles_to_all_subscribers() {
    let recorder = WasmRecorder::builder().buffer_size(8).build().unwrap();
    let time = chrono::DateTime::from_timestamp_micros(1_700_000_000_123_456).unwrap();
    recorder.record_counter_at(time, "before_subscription", 1);
    let mut local = recorder.subscribe();
    let mut export = recorder.subscribe();
    assert_eq!(local.try_recv(), Err(TryRecvError::Empty));
    let key = Key::from_parts(
        String::from("audio_scheduler_delay"),
        vec![metrics::Label::new(
            String::from("participant_id"),
            String::from("alice"),
        )],
    );
    let operations = [
        MetricOperation::IncrementCounter(2),
        MetricOperation::SetCounter(5),
        MetricOperation::IncrementGauge(1.25),
        MetricOperation::DecrementGauge(0.75),
        MetricOperation::SetGauge(4.5),
        MetricOperation::RecordHistogram(0.125),
    ];
    for op in operations {
        recorder.record_at(time, key.clone(), op);
        let expected = Event::Metric {
            time: Some(time),
            key: key.clone(),
            op,
        };
        assert_eq!(local.try_recv().unwrap(), expected);
        assert_eq!(export.try_recv().unwrap(), expected);
    }
    recorder.describe_counter("frames".into(), Some(metrics::Unit::Count), "Rendered frames".into());
    let description = Event::Description {
        name: "frames".into(),
        metric_type: MetricType::Counter,
        unit: Some(metrics::Unit::Count),
        description: "Rendered frames".into(),
    };
    assert_eq!(local.try_recv().unwrap(), description);
    assert_eq!(export.try_recv().unwrap(), description);
    metrics::with_local_recorder(&recorder, || {
        metrics::counter!("frames", "participant_id" => "alice").increment(3);
        metrics::gauge!("queue_depth").set(4.0);
        metrics::histogram!("delay").record(0.5);
    });
    for op in [
        MetricOperation::IncrementCounter(3),
        MetricOperation::SetGauge(4.0),
        MetricOperation::RecordHistogram(0.5),
    ] {
        let first = local.try_recv().unwrap();
        assert_eq!(export.try_recv().unwrap(), first);
        assert!(matches!(first, Event::Metric { time: None, op: actual, .. } if actual == op));
    }
    drop(local);
    recorder.record_counter_at(time, "remaining_subscriber", 7);
    assert!(matches!(
        export.try_recv().unwrap(),
        Event::Metric {
            op: MetricOperation::IncrementCounter(7),
            ..
        }
    ));
    drop(export);
    recorder.record_counter_at(time, "no_subscribers", 8);
    let mut later = recorder.subscribe();
    assert_eq!(later.try_recv(), Err(TryRecvError::Empty));
    recorder.record_counter_at(time, "after_resubscription", 9);
    assert!(matches!(
        later.try_recv().unwrap(),
        Event::Metric {
            op: MetricOperation::IncrementCounter(9),
            ..
        }
    ));
}

#[test]
fn keeps_buffer_limits_and_closes_after_last_recorder() {
    let recorder = WasmRecorder::builder().buffer_size(2).build().unwrap();
    let time = chrono::DateTime::from_timestamp_micros(1_700_000_000_123_456).unwrap();
    let mut fast = recorder.subscribe();
    let mut slow = recorder.subscribe();
    for value in 0..4 {
        recorder.record_counter_at(time, "packets", value);
        assert!(
            matches!(fast.try_recv().unwrap(), Event::Metric { op: MetricOperation::IncrementCounter(actual), .. } if actual == value)
        );
    }
    assert_eq!(slow.try_recv(), Err(TryRecvError::Lagged(2)));
    for value in 2..4 {
        assert!(
            matches!(slow.try_recv().unwrap(), Event::Metric { op: MetricOperation::IncrementCounter(actual), .. } if actual == value)
        );
    }
    let clone = recorder.clone();
    drop(recorder);
    assert_eq!(fast.try_recv(), Err(TryRecvError::Empty));
    drop(clone);
    assert_eq!(fast.try_recv(), Err(TryRecvError::Closed));
    assert_eq!(slow.try_recv(), Err(TryRecvError::Closed));
}
