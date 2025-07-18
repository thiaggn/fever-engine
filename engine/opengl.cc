module;
#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <print>

export module opengl;

namespace gl {

static bool has_started_glfw = false;

void start_glfw(int major, int minor) {
    if (has_started_glfw)
        return;

    if (glfwInit() == GLFW_FALSE) {
        std::println("glfw: falhou em inicializar");
        std::exit(EXIT_FAILURE);
    }

    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, major);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, minor);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
    glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
    has_started_glfw = true;
}

void start_glad() {
    if (gladLoadGL() != 0) {
        std::println("glad: falhou em inicializar glad");
        std::exit(EXIT_FAILURE);
    }
}

export class display {
    GLFWwindow* window;

public:
    display(int width, int height, const char* title) {
        start_glfw(4, 2);

        window = glfwCreateWindow(width, height, title, nullptr, nullptr);
        if (window == nullptr) {
            std::println("glfw: falhou em criar uma janela");
            std::exit(EXIT_FAILURE);
        }

        glfwMakeContextCurrent(window);
    }



    void terminate() const {
        glfwDestroyWindow(window);
        glfwTerminate();
    }

    [[nodiscard]] bool should_close() const {
        return glfwWindowShouldClose(window);
    }

    void swap_buffers() const {
        glfwSwapBuffers(window);
        glfwPollEvents();
    }
};

};