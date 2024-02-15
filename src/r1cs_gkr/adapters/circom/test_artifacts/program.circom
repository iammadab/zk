pragma circom 2.1.4;

// Create a circuit which takes an input 'a',(array of length 2 ) , then  implement power modulo
// and return it using output 'c'.

// HINT: Non Quadratic constraints are not allowed.

template Pow(power) {
   signal input in;
   signal exponentiation_trace[power];
   signal output out;

   exponentiation_trace[0] <== in;

   for (var i = 1; i < power; i++) {
      exponentiation_trace[i] <== exponentiation_trace[i-1] * in;
   }

   out <== exponentiation_trace[power - 1];
}

component main = Pow(4);

/* INPUT = {
    "in": "2"
} */