#!/usr/bin/env perl
use strict;
use warnings;

use Errno qw(EAGAIN EWOULDBLOCK EINTR);
use Fcntl qw(O_RDONLY O_NOCTTY O_NONBLOCK);

sub usage {
    die "usage: phase13-os-native-reader.pl DEVICE\n";
}

@ARGV == 1 or usage();
my ($device) = @ARGV;

sysopen(my $serial, $device, O_RDONLY | O_NOCTTY | O_NONBLOCK)
  or die "open serial device failed: $!\n";

$SIG{INT} = sub { exit 130 };
$SIG{TERM} = sub { exit 143 };

while (1) {
    my $readable = '';
    vec($readable, fileno($serial), 1) = 1;
    my $ready = select($readable, undef, undef, 0.25);
    if (!defined $ready) {
        next if $! == EINTR;
        die "serial select failed: $!\n";
    }
    next if $ready == 0;

    my $bytes_read = sysread($serial, my $buffer, 4096);
    if (!defined $bytes_read) {
        next if $! == EAGAIN || $! == EWOULDBLOCK || $! == EINTR;
        die "serial read failed: $!\n";
    }
    next if $bytes_read == 0;

    print STDOUT $buffer or die "stdout write failed: $!\n";
}
