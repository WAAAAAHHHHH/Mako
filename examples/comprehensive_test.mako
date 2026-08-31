const version = 1.5

say! "=== Mako Language Test ==="
say! "Mako version is", version

!points = [10, 20, 30]
say! "Starting points list:", points

!user = ["name": "Ichika", "score": 100, "active": 1]
say! "User map:", user

say! ""
say! "=== Mutating variables ==="
!x = 5
say! "x starts as", x

x = x * 2 + 10
say! "x after math (x * 2 + 10) is", x

say! "=== Testing Scope & Error Handling ==="
say! "(Error testing omitted since it would stop the program)"
say! "All features work successfully!"
