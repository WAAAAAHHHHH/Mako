say! "=== Stage 3: Data Structures & Modularity ==="

say! ""
say! "--- Indexing: list[i] ---"
!nums = [10, 20, 30, 40, 50]
say! "First element:", nums[0]
say! "Third element:", nums[2]
say! "Last element:", nums[4]

say! ""
say! "--- Index assignment: list[i] = val ---"
nums[0] = 99
say! "After nums[0] = 99:", nums

say! ""
say! "--- Map indexing: map[key] ---"
!person = ["name": "Ichika", "age": 17]
say! "Name:", person["name"]
say! "Age:", person["age"]
person["age"] = 18
say! "After birthday:", person["age"]

say! ""
say! "--- Tuples ---"
!coords = (10, 20)
say! "Coords:", coords
say! "X:", coords[0], "Y:", coords[1]

say! ""
say! "--- Built-in List methods ---"
!items = [3, 1, 2]
say! "Length:", items.len()
say! "First:", items.first()
say! "Last:", items.last()
items = items.push(4)
say! "After push(4):", items
items = items.reverse()
say! "Reversed:", items
say! "Contains 3?", items.contains(3)
say! "Joined:", items.join(", ")

say! ""
say! "--- Built-in Map methods ---"
!scores = ["alice": 95, "bob": 87]
say! "Keys:", scores.keys()
say! "Has alice?", scores.has("alice")
say! "Has charlie?", scores.has("charlie")
say! "Length:", scores.len()

say! ""
say! "--- Built-in String methods ---"
!greeting = "Hello, World!"
say! "Length:", greeting.len()
say! "Upper:", greeting.upper()
say! "Lower:", greeting.lower()
say! "Contains 'World'?", greeting.contains("World")
say! "Starts with 'Hello'?", greeting.starts_with("Hello")
say! "Split by ', ':", greeting.split(", ")

say! ""
say! "=== Stage 4: Types & Error Handling ==="

say! ""
say! "--- Types (Classes) ---"
type Animal begin
    fn init(name, sound) begin
        self.name = name
        self.sound = sound
    end

    fn speak() begin
        give "{self.name} says {self.sound}!"
    end

    fn describe() begin
        give "I am {self.name}"
    end
end

!cat = Animal("Cat", "Meow")
!dog = Animal("Dog", "Woof")
say! cat.speak()
say! dog.speak()
say! cat.describe()
say! "Cat name:", cat.name

say! ""
say! "--- Member assignment ---"
cat.name = "Kitty"
say! "Renamed cat:", cat.name

say! ""
say! "--- Error Handling: try / catch / throw ---"
try begin
    throw "Something went wrong!"
end catch err begin
    say! "Caught error:", err
end

say! ""
say! "--- Error with function ---"
fn safe_divide(a, b) begin
    if b == 0 begin
        throw "Division by zero!"
    end
    give a / b
end

try begin
    !result = safe_divide(10, 2)
    say! "10 / 2 =", result
    !bad = safe_divide(5, 0)
    say! "This should not print"
end catch err begin
    say! "Caught:", err
end

say! ""
say! "=== Stage 5: Advanced Features ==="

say! ""
say! "--- Pattern Matching ---"
!grade = "B"
match grade
case "A" begin
    say! "Excellent!"
end
case "B" begin
    say! "Good job!"
end
case "C" begin
    say! "Average"
end
else begin
    say! "Keep trying!"
end

say! ""
say! "--- Logical Operators: and, or, not ---"
!x = 5
if x > 3 and x < 10 begin
    say! "x is between 3 and 10"
end

if x < 0 or x > 3 begin
    say! "x is less than 0 or greater than 3"
end

if not x == 99 begin
    say! "x is not 99"
end

say! ""
say! "--- true and false literals ---"
!flag = true
if flag begin
    say! "flag is true!"
end
!off = false
if not off begin
    say! "off is false!"
end

say! ""
say! "=== All Stage 3-5 tests passed! ==="
