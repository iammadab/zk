pragma circom 2.1.4;

// Input : 'a',array of length 2 .
// Output : 'c
// Using a forLoop , add a[0] and a[1] , 4 times in a row .

template ForLoop() {
    signal input a[2];
    signal output c;

    signal sum;
    signal repeated_sum[4];

    sum <== a[0] + a[1];
    repeated_sum[0] <== sum;

    for (var i = 1; i <= 3; i++) {
        repeated_sum[i] <== repeated_sum[i - 1] + sum;
    }

    c <== repeated_sum[3];
}

component main = ForLoop();


/* INPUT = {
    "a": ["2", "5"]
} */