import opengl;

int main() {
    gl::display display(800, 450, "Fever");

    while (not display.should_close()) {
        display.swap_buffers();
    }

    display.terminate();
}