function factorial(n) {
    const result = (n==1)?1:n * factorial(n - 1);
    console.log(result);
    return result;
}
factorial(12);
