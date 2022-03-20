function fibonacci(n) {
    const result = (n<=1)?n:fibonacci(n-1) + fibonacci(n - 2);
    return result;
}
console.log(fibonacci(20));
