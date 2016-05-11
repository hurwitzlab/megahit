#!/bin/bash

set -u

WRAPPERDIR=$( cd "$( dirname "$0" )" && pwd )

DEBUG=''
if [[ -n ${VERBOSE:-''} ]]; then
  DEBUG='--debug'
fi

$WRAPPERDIR/run-megahit.pl -d ${IN_DIR:-''} -o ${OUT_DIR:-"${WRAPPERDIR}/velvet-out"} -c ${MIN_COUNT:-''} -n ${K_MIN:-''} -x ${K_MAX:-''} -s ${K_STEP:-''} -l ${K_LIST:-''}
