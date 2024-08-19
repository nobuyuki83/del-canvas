#include <string>
#include <cstdlib>
#include <glad/glad.h>
#define GL_SILENCE_DEPRECATION
#include <GLFW/glfw3.h>
#include <cuda_gl_interop.h>
#include <cuda_runtime.h>
//
#include "util_opengl.h"
#define M_PI 3.1415

// cudaのエラー検出用マクロ
#define EXIT_IF_FAIL(call)                                                 \
  do {                                                                     \
    (call);                                                                \
    cudaError_t err = cudaGetLastError();                                  \
    if (err != cudaSuccess) {                                              \
      std::cout << "error in file " << __FILE__ << " line at " << __LINE__ \
                << ": " << cudaGetErrorString(err) << std::endl;           \
      exit(1);                                                             \
    }                                                                      \
  } while (0)

__global__
void kernel(uchar4 *bitmap, int tick) {
  int x = threadIdx.x + blockIdx.x * blockDim.x;
  int y = threadIdx.y + blockIdx.y * blockDim.y;
  int offset = x + y * blockDim.x * gridDim.x;

  // 連続的になるように...
  float theta = tick / 60.0f * 2.0f * M_PI;
  float theta_x = x / 60.0f * 2.0f * M_PI;
  float theta_y = y / 60.0f * 2.0f * M_PI;
  float r = fabs(sin(theta + theta_x));
  float g = fabs(cos(theta + theta_y));
  float b = fabs(sin(theta + theta_x) * cos(theta + theta_y));

  bitmap[offset].x = (unsigned char)(r * 255);
  bitmap[offset].y = (unsigned char)(g * 255);
  bitmap[offset].z = (unsigned char)(b * 255);
  bitmap[offset].w = 255;
}

// フレームバッファの取得に使用
cudaGraphicsResource *dev_resource;

#define WIDTH 1024
#define HEIGHT 1024

int main() {
  if (!glfwInit()) { exit(EXIT_FAILURE); }
  // set OpenGL's version (note: ver. 2.1 is very old, but I chose because it's simple)
  ::glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 2);
  ::glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 1);
  GLFWwindow *window = ::glfwCreateWindow(WIDTH, HEIGHT, "task05", nullptr, nullptr);
  if (!window) { // exit if failed to create window
    ::glfwTerminate();
    exit(EXIT_FAILURE);
  }
  ::glfwMakeContextCurrent(window); // working on this window below
  //
  if (!gladLoadGL()) {     // glad: load all OpenGL function pointers
    printf("Something went wrong in loading OpenGL functions!\n");
    exit(-1);
  }

  int shaderProgram;
  {
    const auto vrt_path = std::string(SOURCE_DIR) + "/shader.vert";
    const auto frg_path = std::string(SOURCE_DIR) + "/shader.frag";
    std::string vrt = acg::load_file_as_string(vrt_path.c_str()); // read source code of vertex shader program
    std::string frg = acg::load_file_as_string(frg_path.c_str()); // read source code of fragment shader program
    shaderProgram = acg::create_shader_program(vrt, frg); // compile the shader on GPU
  }

  glDisable(GL_MULTISAMPLE);
  const GLint iloc = glGetUniformLocation(shaderProgram, "time");  // location of variable in the shader program

  //::glClearColor(1, 1, 1, 1);  // set the color to fill the frame buffer when glClear is called.
  //::glEnable(GL_DEPTH_TEST);

    GLuint pbo;
    // バッファを作成
    glGenBuffers(1, &pbo);
    glBindBuffer(GL_PIXEL_UNPACK_BUFFER, pbo);
    glBufferData(GL_PIXEL_UNPACK_BUFFER,
                 sizeof(char4) * WIDTH * HEIGHT,
                 NULL,
                 GL_DYNAMIC_DRAW);

    // OpenGLのバッファをCudaと共有する設定
    EXIT_IF_FAIL(cudaGraphicsGLRegisterBuffer(
        &dev_resource, pbo, cudaGraphicsMapFlagsNone));

    std::cout << dev_resource << std::endl;

  int tick = 0;  // 今何フレーム目?
  while (!::glfwWindowShouldClose(window)) {
    uchar4 *dev_bitmap;
    size_t size;
    EXIT_IF_FAIL(cudaGraphicsMapResources(1, &dev_resource, NULL));
    EXIT_IF_FAIL(cudaGraphicsResourceGetMappedPointer(
          (void **)&dev_bitmap, &size, dev_resource));

    // カーネル関数を呼ぶ
    dim3 threads(8, 8);                 // 64スレッド/1グリッド
    dim3 grids(WIDTH / 8, HEIGHT / 8);  // 各ピクセルに1スレッドが割り振られる
    kernel<<<grids, threads>>>(dev_bitmap, tick);

    // カーネル関数の終了を待つ
    EXIT_IF_FAIL(cudaDeviceSynchronize());

    // リソースの開放
    EXIT_IF_FAIL(cudaGraphicsUnmapResources(1, &dev_resource, NULL));

    ++tick;

/*
    ::glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    const auto time = static_cast<float>(glfwGetTime());
    ::glUniform1f(iloc,time);
    ::glMatrixMode(GL_PROJECTION);
    ::glLoadIdentity(); // identity transformation
    ::glMatrixMode(GL_MODELVIEW);
    ::glLoadIdentity(); // identity transformation
    ::glUseProgram(shaderProgram);  // use the shader program from here
    ::glBegin(GL_QUADS); // draw a rectangle that cover the entire screen
    ::glVertex2d(-1,-1);
    ::glVertex2d(+1,-1);
    ::glVertex2d(+1,+1);
    ::glVertex2d(-1,+1);
    ::glEnd();
*/
    glDrawPixels(WIDTH, HEIGHT, GL_RGBA, GL_UNSIGNED_BYTE, 0);
    ::glfwSwapBuffers(window);
    ::glfwPollEvents();
  }
  // リソースの開放(glutMainLoop()は返らないので、実際は呼ばれない)
  glBindBuffer(GL_PIXEL_UNPACK_BUFFER, 0);
  glDeleteBuffers(1, &pbo);
  EXIT_IF_FAIL(cudaGLUnregisterBufferObject(pbo));
  EXIT_IF_FAIL(cudaGraphicsUnregisterResource(dev_resource));
  //
  ::glfwDestroyWindow(window);
  ::glfwTerminate();
  exit(EXIT_SUCCESS);
}


