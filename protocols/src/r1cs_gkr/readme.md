## R1CS to GKR Circuit

### Term

---
A term is a variable multiplied by some constant e.g. 2a, 5b, 14c, d (when constant equals 1).

### Constraint

---
A constraint is an equation involving one or more terms 
e.g a + 2b = 5c + d. The constraint is satisfied if there exists values for a, b, c and d such that the LHS == RHS.

### Reduced Constraints

---
Reduced constraints have exactly 3 terms and a single operation (addition or subtraction). Reduced constraints are of the following form:

term_a (op) term_b = term_c  (where op is either + or x)

### Constraint to Reduced Constraint(s)

---
TODO


### Reduced Constraint as a Circuit

---
Recall each term is the product of some constant and a variable

term_a (op) term_b = term_c   is equivalent to:

const_a * var_a (op) const_b * var_b  = const_c * var_b

The circuit needs to check that the relationship above holds:

```text
                                   +                               // output layer
                    /                              \
                  OP                                 x             // compute layer
            /            \                     /             \
          x               x                   x               x    // product layer
      /      \        /       \           /        \       /     \
 const_a   var_a const_b    var_b    const_c    var_c     1      -1
```

If the output of the circuit is 0 then the constraint has been satisfied. 

### Dealing with multiple reduced constraints

---
Each reduced constraint is converted into a circuit and each circuit is laid side by side in a row. Leading to a circuit that 
has n outputs (where n is the number of constraints).

All constraints are satisfied if all outputs are zeros. GKR is used to prove this combined circuit with an additional zero check during verification.
