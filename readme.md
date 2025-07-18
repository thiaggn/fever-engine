CMake é usado para gerar os arquivos de build do projeto. O arquivo Makefile contém apenas os comandos para gerenciar o projeto; é só uma conveniência, não é responsável pela compilação.

Para compilar o projeto, use `make compile`. Para executar, `make run`. O projeto precisa do compilador Clang 20.1.7 para ser compilado. Se estiver no Windows, terá que instalar também o MSVC.