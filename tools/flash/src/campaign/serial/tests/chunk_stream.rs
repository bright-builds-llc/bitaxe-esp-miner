use super::*;

#[test]
fn chunk_stream_larger_than_sixteen_mib_reaches_terminal_marker_with_bounded_artifacts() {
    // Arrange
    let mut analyzer = CampaignSerialAnalyzer::new(observation_admission());
    let chunk = vec![b'x'; 64 * 1_024];

    // Act
    for _ in 0..257 {
        assert!(!analyzer.observe_chunk(&chunk));
        assert!(!analyzer.observe_chunk(b"\n"));
    }
    analyzer.observe_chunk(&observation_marker(CAMPAIGN_MARKER_SCHEMA));
    let capture = analyzer.finish();
    let aggregate_bytes = serde_json::to_vec(&capture.aggregate).expect("aggregate JSON");
    let diagnostic_bytes = serde_json::to_vec(&capture.diagnostics).expect("diagnostic JSON");

    // Assert
    assert!(capture.diagnostics.total_bytes > 16 * 1_024 * 1_024);
    assert_eq!(capture.aggregate.marker_count, 1);
    assert!(capture.aggregate.terminal.is_some());
    assert!(aggregate_bytes.len() < 64 * 1_024);
    assert!(diagnostic_bytes.len() < 64 * 1_024);
}
