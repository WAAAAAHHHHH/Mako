say! "=== Control Flow Test ==="
!count = 3

while count begin
    say! "Count is", count
    count = count - 1
end

say! "Count finished!"

!test_val = 1
if test_val begin
    say! "This is true!"
end else begin
    say! "This is false!"
end
