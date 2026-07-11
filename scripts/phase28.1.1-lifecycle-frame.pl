#!/usr/bin/env perl

use strict;
use warnings;

use Errno qw(EINTR);
use Fcntl qw(O_CREAT O_EXCL O_WRONLY);
use IO::Socket::UNIX;
use JSON::PP;
use Socket qw(SOCK_STREAM);

use constant MAX_FRAME_BYTES => 4096;
use constant MAX_LENGTH_DIGITS => 4;
use constant MAX_SOCKET_PATH_BYTES => 103;

umask 0177;

my $created_socket_path;

sub fail_closed {
    my ($category) = @_;
    print STDERR "lifecycle_frame_error=$category\n";
    exit 1;
}

sub cleanup_socket {
    return if !defined $created_socket_path;
    unlink $created_socket_path if -S $created_socket_path;
    undef $created_socket_path;
}

$SIG{INT} = sub {
    cleanup_socket();
    exit 130;
};
$SIG{TERM} = sub {
    cleanup_socket();
    exit 143;
};
END { cleanup_socket() }

sub parse_options {
    my (@arguments) = @_;
    my %options;

    while (@arguments) {
        my $name = shift @arguments;
        fail_closed("invalid_arguments") if $name !~ /^--(?:socket|output)$/;
        fail_closed("invalid_arguments") if !@arguments;
        fail_closed("invalid_arguments") if exists $options{$name};
        $options{$name} = shift @arguments;
    }

    return %options;
}

sub require_socket_path {
    my ($socket_path) = @_;
    fail_closed("invalid_arguments") if !defined $socket_path || $socket_path eq q{};
    fail_closed("socket_path_invalid") if bytes::length($socket_path) > MAX_SOCKET_PATH_BYTES;
}

sub read_one_byte {
    my ($handle) = @_;

    while (1) {
        my $read = sysread($handle, my $byte, 1);
        next if !defined($read) && $! == EINTR;
        fail_closed("invalid_frame") if !defined($read) || $read == 0;
        return $byte;
    }
}

sub read_frame_length {
    my ($handle) = @_;
    my $digits = q{};

    while (1) {
        my $byte = read_one_byte($handle);
        last if $byte eq "\n";
        fail_closed("invalid_frame") if $byte !~ /^[0-9]$/;
        $digits .= $byte;
        fail_closed("invalid_frame") if bytes::length($digits) > MAX_LENGTH_DIGITS;
    }

    fail_closed("invalid_frame") if $digits !~ /^[1-9][0-9]*$/;
    my $length = 0 + $digits;
    fail_closed("invalid_frame") if $length < 1 || $length > MAX_FRAME_BYTES;
    return $length;
}

sub read_exact_payload {
    my ($handle, $length) = @_;
    my $payload = q{};

    while (bytes::length($payload) < $length) {
        my $remaining = $length - bytes::length($payload);
        my $read = sysread($handle, my $fragment, $remaining);
        next if !defined($read) && $! == EINTR;
        fail_closed("invalid_frame") if !defined($read) || $read == 0;
        $payload .= $fragment;
    }

    return $payload;
}

sub require_stream_eof {
    my ($handle) = @_;

    while (1) {
        my $read = sysread($handle, my $extra, 1);
        next if !defined($read) && $! == EINTR;
        fail_closed("invalid_frame") if !defined($read) || $read != 0;
        return;
    }
}

sub require_json_object {
    my ($payload) = @_;
    my $decoded = eval { JSON::PP->new->utf8->decode($payload) };
    fail_closed("invalid_frame") if $@ || ref($decoded) ne "HASH";
}

sub read_stdin_payload {
    my $payload = q{};

    while (1) {
        my $remaining = MAX_FRAME_BYTES + 1 - bytes::length($payload);
        fail_closed("invalid_frame") if $remaining <= 0;
        my $read = sysread(STDIN, my $fragment, $remaining);
        next if !defined($read) && $! == EINTR;
        fail_closed("invalid_frame") if !defined($read);
        last if $read == 0;
        $payload .= $fragment;
        fail_closed("invalid_frame") if bytes::length($payload) > MAX_FRAME_BYTES;
    }

    fail_closed("invalid_frame") if bytes::length($payload) < 1;
    require_json_object($payload);
    return $payload;
}

sub write_all {
    my ($handle, $bytes, $failure_category) = @_;
    my $offset = 0;

    while ($offset < bytes::length($bytes)) {
        my $written = syswrite($handle, $bytes, bytes::length($bytes) - $offset, $offset);
        next if !defined($written) && $! == EINTR;
        fail_closed($failure_category) if !defined($written) || $written == 0;
        $offset += $written;
    }
}

sub write_private_output {
    my ($output_path, $payload) = @_;
    fail_closed("invalid_arguments") if !defined $output_path || $output_path eq q{};
    sysopen(my $output, $output_path, O_WRONLY | O_CREAT | O_EXCL, 0600)
      or fail_closed("output_unavailable");
    chmod 0600, $output_path or fail_closed("output_unavailable");
    write_all($output, $payload, "output_unavailable");
    close $output or fail_closed("output_unavailable");
}

sub receive_frame {
    my (%options) = @_;
    my $socket_path = $options{"--socket"};
    my $output_path = $options{"--output"};
    require_socket_path($socket_path);
    fail_closed("invalid_arguments") if !defined $output_path;
    fail_closed("socket_unavailable") if -e $socket_path;

    my $server = IO::Socket::UNIX->new(
        Type   => SOCK_STREAM,
        Local  => $socket_path,
        Listen => 1,
    ) or fail_closed("socket_unavailable");
    $created_socket_path = $socket_path;
    chmod 0600, $socket_path or fail_closed("socket_unavailable");

    my $client = $server->accept or fail_closed("socket_unavailable");
    my $length = read_frame_length($client);
    my $payload = read_exact_payload($client, $length);
    require_stream_eof($client);
    require_json_object($payload);
    close $client or fail_closed("socket_unavailable");
    close $server or fail_closed("socket_unavailable");
    cleanup_socket();
    write_private_output($output_path, $payload);
}

sub send_frame {
    my (%options) = @_;
    my $socket_path = $options{"--socket"};
    require_socket_path($socket_path);
    fail_closed("invalid_arguments") if exists $options{"--output"};
    my $payload = read_stdin_payload();
    my $socket = IO::Socket::UNIX->new(
        Type => SOCK_STREAM,
        Peer => $socket_path,
    ) or fail_closed("socket_unavailable");
    my $frame = bytes::length($payload) . "\n" . $payload;
    write_all($socket, $frame, "socket_unavailable");
    shutdown($socket, 1) or fail_closed("socket_unavailable");
    close $socket or fail_closed("socket_unavailable");
}

my $command = shift @ARGV // q{};
my %options = parse_options(@ARGV);

if ($command eq "receive") {
    fail_closed("invalid_arguments") if !exists $options{"--socket"} || !exists $options{"--output"};
    receive_frame(%options);
    exit 0;
}

if ($command eq "send") {
    fail_closed("invalid_arguments") if !exists $options{"--socket"};
    send_frame(%options);
    exit 0;
}

fail_closed("invalid_arguments");
