#!/usr/bin/env bash

label=$1
duration=$2 # sec
step=$3 # sec

t=0
while [ $t -lt ${duration} ]; do
    printf "%s %d\n" ${label} $t
    t=$(($t+${step}))
    sleep ${step}
done
printf "%s done.\n" ${label}
