#!/usr/bin/env bash

label=$1

i=1
while [ $i -le 3 ]; do
    t=0
    printf "%s-%d " ${label} $i
    while [ $t -lt 10 ]; do
        printf "%d" $t
        ((t++))
        sleep 1
    done
    printf "\n"
    ((i++))
done
