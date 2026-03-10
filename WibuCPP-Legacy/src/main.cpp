#include <iostream>
#include <fstream>
#include <string>
#include <sstream>
#include "lexer.hpp"
#include "interpreter.hpp"

int main(int argc, char *argv[])
{
    Interpreter interp; 

    if (argc == 2 && std::string(argv[1]) == "--repl")
    {
        std::cout << "Wibu Shell v0.1 (WIBU REPL)\n";
        std::cout << "Type 'exit' or 'quit' to exit, or Ctrl+D (Unix) / Ctrl+Z (Windows) + Enter\n";
        std::cout << ">>> ";

        std::string line;
        while (std::getline(std::cin, line))
        {
            if (line == "exit" || line == "quit")
            {
                std::cout << "Bye!\n";
                break;
            }

            if (line.empty())
            {
                std::cout << ">>> ";
                continue;
            }

            Lexer lexer(line);
            auto tokens = lexer.tokenize();

            if (tokens.empty())
            {
                std::cout << ">>> ";
                continue;
            }

            try
            {
                auto result = interp.eval(tokens);

                if (!result.hasReturn && result.returnValue.type != WB_NONE)
                {
                    std::cout << result.returnValue.toString() << std::endl;
                }
            }
            catch (const std::exception &e)
            {
                std::cerr << "Runtime Error: " << e.what() << std::endl;
            }
            catch (...)
            {
                std::cerr << "Unknown runtime error" << std::endl;
            }

            std::cout << ">>> ";
        }

        return 0;
    }

    if (argc < 2)
    {
        std::cerr << "Daftar tutor:\n";
        std::cerr << "  " << argv[0] << " <source_file>     : Jalananin script wibu\n";
        std::cerr << "  " << argv[0] << " --repl            : Jalanan REPL\n";
        return 1;
    }

    std::fstream file(argv[1]);
    if (!file.is_open())
    {
        std::cerr << "Error: Yah error sih lu '" << argv[1] << "'" << std::endl;
        return 1;
    }

    std::string code((std::istreambuf_iterator<char>(file)),
                     std::istreambuf_iterator<char>());
    file.close();

    Lexer lexer(code);
    auto tokens = lexer.tokenize();

    try
    {
        interp.eval(tokens);
    }
    catch (const std::exception &e)
    {
        std::cerr << "Runtime Error: " << e.what() << std::endl;
        return 1;
    }
    catch (...)
    {
        std::cerr << "Unknown runtime error" << std::endl;
        return 1;
    }

    return 0;
}