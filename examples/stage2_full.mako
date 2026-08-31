say! "=== Stage 2: Full Test Suite ==="
say! ""

say! "--- Functions & give ---"
fn greet(name) begin
    give "Hello, {name}!"
end

fn add(a, b) begin
    give a + b
end

fn factorial(n) begin
    if n <= 1 begin
        give 1
    end
    give n * factorial(n - 1)
end

say! greet("Mako")
say! "3 + 7 =", add(3, 7)
say! "5! =", factorial(5)

say! ""
say! "--- elif chains ---"
!score = 75

if score >= 90 begin
    say! "Grade: A"
end elif score >= 75 begin
    say! "Grade: B"
end elif score >= 60 begin
    say! "Grade: C"
end else begin
    say! "Grade: F"
end

say! ""
say! "--- for loops ---"
!nums = [1, 2, 3, 4, 5]

for n in nums begin
    say! "item:", n
end

say! ""
say! "--- stop and skip ---"
!i = 0
while i < 10 begin
    i = i + 1
    if i == 3 begin
        skip
    end
    if i == 6 begin
        stop
    end
    say! "i =", i
end

say! ""
say! "=== All Stage 2 tests passed! ==="
