Dependencias:
* [CMake](https://cmake.org/download/) para gerar o script de build do projeto
* [Ninja](https://ninja-build.org/) para executar o script de build (Ninja é uma alternativa ao Makefile)
* Makefile para executar comandos convenientes
* [Clang 20.1.7](https://github.com/llvm/llvm-project/releases/tag/llvmorg-20.1.8) para compilar o projeto
* Se estiver no Windows, será necessário o Visual Studio 2022 junto ao Clang.

O arquivo Makefile contém apenas os comandos para gerenciar o projeto; é só uma conveniência. Por exemplo, para compilar e em seguida executar, digite `make run` na linha de comando.