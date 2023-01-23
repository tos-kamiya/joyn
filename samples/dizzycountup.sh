#!/usr/bin/env bash

msg=$1
duration=$2 # sec

t=0
while [ ${t} -lt ${duration} ]; do
    printf "%s %d\n" ${msg} ${t}
    v=$((1+$RANDOM%3))
    t=$(($t+$v))
    sleep $v
done
printf "%s %d\n" ${msg} ${t}
