Dependencias:
* CMake para gerar o script de build do projeto
* Ninja para executar o script de build (não uso Makefile)
* Makefile apenas para executar comandos convenientes
* Clang 20.1.7 para compilar o projeto
* Se estiver no Windows, será necessário o Visual Studio 2022 junto ao Clang.

O arquivo Makefile contém apenas os comandos para gerenciar o projeto; é só uma conveniência. Por exemplo, para compilar e em seguida executar, digite `make run` na linha de comando.