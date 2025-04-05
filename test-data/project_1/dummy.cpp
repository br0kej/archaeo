#include <iostream>
#include <vector>
#include <string>
#include <memory>
#include <algorithm>
#include <map>

// Forward declarations
class Animal;
class Dog;

// Abstract base class
class Animal {
protected:
    std::string name;
    int age;

public:
    Animal(const std::string& name, int age) : name(name), age(age) {}
    virtual ~Animal() = default;

    virtual void makeSound() const = 0;

    std::string getName() const { return name; }
    int getAge() const { return age; }
};

// Derived class
class Dog : public Animal {
private:
    std::string breed;

public:
    Dog(const std::string& name, int age, const std::string& breed)
        : Animal(name, age), breed(breed) {}

    void makeSound() const override {
        std::cout << "Woof! I'm a " << breed << " named " << name << std::endl;
    }

    std::string getBreed() const { return breed; }
};

// Template class
template<typename T>
class Container {
private:
    std::vector<T> data;

public:
    void add(const T& item) {
        data.push_back(item);
    }

    bool remove(const T& item) {
        auto it = std::find(data.begin(), data.end(), item);
        if (it != data.end()) {
            data.erase(it);
            return true;
        }
        return false;
    }

    size_t size() const {
        return data.size();
    }

    const std::vector<T>& getData() const {
        return data;
    }
};

// Smart pointer example class
class Resource {
private:
    std::string data;

public:
    explicit Resource(const std::string& d) : data(d) {
        std::cout << "Resource constructed with: " << data << std::endl;
    }

    ~Resource() {
        std::cout << "Resource destroyed: " << data << std::endl;
    }

    void use() const {
        std::cout << "Using resource: " << data << std::endl;
    }
};

// Function template
template<typename T>
T max_value(T a, T b) {
    return (a > b) ? a : b;
}

// Lambda function example
auto sort_by_length = [](const std::string& a, const std::string& b) {
    return a.length() < b.length();
};

int main() {
    // Smart pointer usage
    std::cout << "\n=== Smart Pointer Example ===" << std::endl;
    {
        auto resource = std::make_unique<Resource>("Important Data");
        resource->use();
    } // Resource automatically destroyed here

    // Polymorphism example
    std::cout << "\n=== Polymorphism Example ===" << std::endl;
    std::vector<std::unique_ptr<Animal>> animals;
    animals.push_back(std::make_unique<Dog>("Buddy", 3, "Labrador"));
    animals.push_back(std::make_unique<Dog>("Max", 5, "German Shepherd"));

    for (const auto& animal : animals) {
        animal->makeSound();
    }

    // Container template example
    std::cout << "\n=== Container Template Example ===" << std::endl;
    Container<int> numbers;
    numbers.add(1);
    numbers.add(2);
    numbers.add(3);

    std::cout << "Container size: " << numbers.size() << std::endl;

    // STL algorithms and lambda example
    std::cout << "\n=== STL and Lambda Example ===" << std::endl;
    std::vector<std::string> words = {"cat", "elephant", "dog", "hippopotamus"};
    std::sort(words.begin(), words.end(), sort_by_length);

    std::cout << "Sorted words by length:" << std::endl;
    for (const auto& word : words) {
        std::cout << word << std::endl;
    }

    // Map example
    std::cout << "\n=== Map Example ===" << std::endl;
    std::map<std::string, int> scores = {
        {"Alice", 95},
        {"Bob", 87},
        {"Charlie", 92}
    };

    for (const auto& [name, score] : scores) {
        std::cout << name << ": " << score << std::endl;
    }

    // Template function example
    std::cout << "\n=== Template Function Example ===" << std::endl;
    std::cout << "Max of 10 and 20: " << max_value(10, 20) << std::endl;
    std::cout << "Max of 3.14 and 2.718: " << max_value(3.14, 2.718) << std::endl;

    return 0;
}