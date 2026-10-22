use super::*;

fn fifo() -> (tempfile::TempDir, PrivateFifo) {
    let parent = tempfile::tempdir().expect("parent");
    let parent_path = Utf8PathBuf::from_path_buf(parent.path().to_path_buf()).expect("utf-8");
    let fifo = PrivateFifo::create(&parent_path, "data.fifo").expect("fifo");
    (parent, fifo)
}

#[test]
fn a_tool_s_output_arrives_in_memory() {
    // Arrange
    let (_parent, fifo) = fifo();
    let path = fifo.path().to_owned();
    // Act
    let bytes = read_while(&fifo, 64, || {
        fs::write(path.as_std_path(), b"partition bytes")?;
        Ok(())
    })
    .expect("read");
    // Assert
    assert_eq!(bytes, b"partition bytes");
}

#[test]
fn the_pipe_is_not_a_regular_file() {
    // Arrange
    let (_parent, fifo) = fifo();
    // Act
    let metadata = fs::symlink_metadata(fifo.path().as_std_path()).expect("metadata");
    // Assert
    assert!(!metadata.is_file());
}

#[test]
fn a_tool_that_never_writes_does_not_hang_the_reader() {
    // Arrange
    let (_parent, fifo) = fifo();
    // Act
    let bytes = read_while(&fifo, 64, || Ok(())).expect("released");
    // Assert
    assert!(bytes.is_empty());
}

#[test]
fn a_failing_tool_s_error_is_returned() {
    // Arrange
    let (_parent, fifo) = fifo();
    // Act
    let error = read_while(&fifo, 64, || bail!("tool failed")).expect_err("tool error");
    // Assert
    assert!(error.to_string().contains("tool failed"));
}

#[test]
fn output_past_the_limit_is_refused() {
    // Arrange
    let (_parent, fifo) = fifo();
    let path = fifo.path().to_owned();
    // Act
    let error = read_while(&fifo, 4, || {
        fs::write(path.as_std_path(), b"too many bytes")?;
        Ok(())
    })
    .expect_err("oversize");
    // Assert
    assert!(error.to_string().contains("oversize"));
}

#[test]
fn bytes_are_fed_to_a_tool_through_the_pipe() {
    // Arrange
    let (_parent, fifo) = fifo();
    let path = fifo.path().to_owned();
    // Act
    let seen = feed_while(&fifo, b"nvs image".to_vec(), || {
        Ok(fs::read(path.as_std_path())?)
    })
    .expect("fed");
    // Assert
    assert_eq!(seen, b"nvs image");
}

#[test]
fn a_tool_that_never_reads_does_not_hang_the_writer() {
    // Arrange
    let (_parent, fifo) = fifo();
    // Act
    let result = feed_while(&fifo, b"nvs image".to_vec(), || Ok(()));
    // Assert: the writer is released; a broken pipe on the unread bytes is reported.
    assert!(result.is_ok() || result.unwrap_err().to_string().contains("memory_fifo"));
}

#[test]
fn the_private_directory_is_removed_when_dropped() {
    // Arrange
    let (_parent, fifo) = fifo();
    let directory = fifo.path().parent().expect("directory").to_owned();
    // Act
    drop(fifo);
    // Assert
    assert!(fs::symlink_metadata(directory.as_std_path()).is_err());
}

#[test]
fn a_tool_that_fails_without_reading_reports_its_own_error() {
    // Arrange: more than a pipe buffer, so an unread write would break the pipe.
    let (_parent, fifo) = fifo();
    let image = vec![0_u8; 0x6000];
    // Act
    let error = feed_while(&fifo, image, || -> Result<()> { bail!("nvs_tool failed") })
        .expect_err("tool error");
    // Assert
    assert!(error.to_string().contains("nvs_tool failed"), "{error}");
}
