# thu-learn-helper

You can regard this project as a Rust version of [thu-learn-lib](https://github.com/Harry-Chen/thu-learn-lib) by [Harry Chen](https://github.com/Harry-Chen), though there are some differences between these two projects in detail.

Currently it only supports interacting with web-learning as a student, not a teacher or a TA. I may or may not add this feature in the future.

By default all the apis are `async`. By enabling `featues = ["blocking"]`, you will get a set of blocking apis.

# Usage

You can refer to `examples/example.rs`, which reads username and password from stdin, login and print the information of all the classes in the current semester.

All the functions and types come with detailed documentation (maybe somewhat wordy), and thus if you are still confused about the usage, you can fire an issue to me.