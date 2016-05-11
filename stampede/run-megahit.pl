#!/usr/bin/env perl

use strict;
use warnings;
use feature 'say';
use autodie;
use Cwd 'cwd';
use Data::Dump 'dump';
use FindBin '$Bin';
use File::Basename 'fileparse';
use File::Spec::Functions qw'catdir catfile';
use File::Path 'remove_tree';
use File::Find::Rule;
use Getopt::Long;
use Pod::Usage;
use Readonly;

my $DEBUG = 0;

main();

# --------------------------------------------------
sub main {
    my %args = get_args();

    if ($args{'help'} || $args{'man_page'}) {
        pod2usage({
            -exitval => 0,
            -verbose => $args{'man_page'} ? 2 : 1
        });
    }; 

    debug("args = ", dump(\%args));
    my $in_dir  = $args{'dir'}     or pod2usage('No input --dir');
    my $out_dir = $args{'out_dir'} or pod2usage('No --out_dir');
    my $megahit = $args{'megahit'} or pod2usage('No --megahit');

    unless (-s $megahit) {
        pod2usage("Cannot find binary ($megahit)");
    }

    my @files;
    if (-f $in_dir) {
        debug("Directory arg '$in_dir' is actually a file");
        push @files, $in_dir;
    }
    else {
        debug("Looking for files in '$in_dir'");
        @files = File::Find::Rule->file()->in($in_dir);
    }

    printf "Found %s files\n", scalar(@files);

    if (@files < 1) {
        pod2usage('No input data');
    }

    my @inputs;
    for my $file (@files) {
        my ($basename, $path, $ext) = fileparse($file, qr/\.[^.]*/);
        $ext =~ s/^\.//; # remove leading dot
        $ext = lc $ext;  

        my $type = '-r';
        if ($basename =~ /[_.]r([12])[_.]/) {
            $type = '-' . $1;
        }
        elsif ($basename =~ /[_.]paired[_.]/) {
            $type = '--12';
        }

        unless ($type) {
            pod2usage("Can't figure type (-1, -2, --12, -r) of '$file'");
        }

        push @inputs, { file => $file, format => $type };
    }   

    unless (@inputs) {
        pod2usage("Found no usable inputs");
    } 

    debug("inputs =", dump(\@inputs));

    my @options;
    for my $opt (qw[min_count k_min k_max k_step k_list]) {
        if (my $val = $args{ $opt }) {
            my $name = $opt;
            $name =~ s/_/-/g;
            push @options, sprintf "--%s %s", $name, $val;
        }
    }

    execute(join(' ',
        $megahit,
        @options,
        '-o', $args{'out_dir'}, 
        join(' ', map { sprintf '%s %s', $_->{'format'}, $_->{'file'} } @inputs)
    ));

    printf("Finished, see results in '%s'\n", $args{'out_dir'});
}

# --------------------------------------------------
sub debug {
    say @_ if $DEBUG;
}

# --------------------------------------------------
sub execute {
    my @cmd = @_ or return;
    debug("\n\n>>>>>>\n\n", join(' ', @cmd), "\n\n<<<<<<\n\n");

    unless (system(@cmd) == 0) {
        die sprintf(
            "FATAL ERROR! Could not execute command:\n%s\n",
            join(' ', @cmd)
        );
    }
}

# --------------------------------------------------
sub get_args {
    my %args = (
        'dir'       => '',
        'debug'     => 0,
        'min_count' => 2,
        'k_min'     => 21,
        'k_max'     => 99,
        'k_step'    => 20,
        'k_list'    => '',
        'out_dir'   => catdir(cwd(), 'megahit-out'),
        'megahit'   => catfile($Bin, 'bin', 'megahit'),
    );

    GetOptions(
        \%args,
        'dir|d=s',
        'min_count|c:i',
        'k_min|n:i',
        'k_max|x:i',
        'k_step|s:i',
        'k_list|l:i',
        'out_dir|o:s',
        'megahit|m:s',
        'debug',
        'help',
        'man',
    ) or pod2usage(2);

    $DEBUG = $args{'debug'};

    if (-d $args{'out_dir'}) {
        remove_tree($args{'out_dir'});
    }

    return %args;
}

__END__

# --------------------------------------------------

=pod

=head1 NAME

run-megahit.pl - runs megahit

=head1 SYNOPSIS

  run-megahit.pl -d /path/to/data

Required Arguments:

  -d|--dir   Input directory

Options (defaults in parentheses):

  -c|--min_count    Minimum multiplicity for filtering (k_min+1)-mers
  -n|--k_min        Minimum kmer size (<= 255), must be odd number
  -x|--k_max        Maximum kmer size (<= 255), must be odd number
  -s|--k_step       Increment of kmer size of each iteration 
                    (<= 28), must be even number
  -l|--k_list       Comma-separated list of kmer size (all must be odd, 
                    in the range 15-255, increment <= 28); 
                    override --k-min, --k-max and --k-step
  -m|--megahit      Path to MEGAHIT binary

  --help            Show brief help and exit
  --man             Show full documentation

=head1 DESCRIPTION

Runs MEGAHIT.

=head1 SEE ALSO

MEGAHIT.

=head1 AUTHOR

Ken Youens-Clark E<lt>kyclark@email.arizona.eduE<gt>.

=head1 COPYRIGHT

Copyright (c) 2016 Ken Youens-Clark

This module is free software; you can redistribute it and/or
modify it under the terms of the GPL (either version 1, or at
your option, any later version) or the Artistic License 2.0.
Refer to LICENSE for the full license text and to DISCLAIMER for
additional warranty disclaimers.

=cut
