#pragma once

#include "token.hpp"
#include <vector>
#include <string>
#include <cctype>
#include <sstream>
#include <unordered_map>

/*lexer*/
class Lexer
{
public:
    Lexer(const std::string &src) : source(src), pos(0) {}

    std::vector<Token> tokenize()
    {
        std::vector<Token> tokens;
        while (pos < source.size())
        {
            char c = source[pos];
            if (isspace(c))
            {
                pos++;
                continue;
            }
            if (c == '/' && pos + 1 < source.size() && source[pos + 1] == '/')
            {
                pos += 2; 
                while (pos < source.size() && source[pos] != '\n')
                {
                    pos++;
                }
                continue;
            }
            if (isdigit(c))
            {
                tokens.push_back(readNumber());
                continue;
            }
            if (isalpha(c))
            {
                tokens.push_back(readIdentifierOrKeyword());
                continue;
            }
            if (c == '"' || c == '\'')
            {
                tokens.push_back(readString());
                continue;
            }

            if (ispunct(c))
            {
                if (pos + 1 < source.size())
                {
                    std::string twoChars = source.substr(pos, 2);
                    if (twoChars == "==")
                    {
                        tokens.push_back({TOKEN_EQ, "=="});
                        pos += 2;
                        continue;
                    }
                    else if (twoChars == "!=")
                    {
                        tokens.push_back({TOKEN_NEQ, "!="});
                        pos += 2;
                        continue;
                    }
                    else if (twoChars == "<=")
                    {
                        tokens.push_back({TOKEN_LE, "<="});
                        pos += 2;
                        continue;
                    }
                    else if (twoChars == ">=")
                    {
                        tokens.push_back({TOKEN_GE, ">="});
                        pos += 2;
                        continue;
                    }
                    else if (twoChars == "&&")
                    {
                        tokens.push_back({TOKEN_AND, "&&"});
                        pos += 2;
                        continue;
                    }
                    else if (twoChars == "||")
                    {
                        tokens.push_back({TOKEN_OR, "||"});
                        pos += 2;
                        continue;
                    }
                }

                if (c == '+')
                {
                    tokens.push_back({TOKEN_PLUS, "+"});
                    pos++;
                    continue;
                }
                else if (c == '-')
                {
                    tokens.push_back({TOKEN_MINUS, "-"});
                    pos++;
                    continue;
                }
                else if (c == '*')
                {
                    tokens.push_back({TOKEN_STAR, "*"});
                    pos++;
                    continue;
                }
                else if (c == '/')
                {
                    tokens.push_back({TOKEN_SLASH, "/"});
                    pos++;
                    continue;
                }
                else if (c == '<')
                {
                    tokens.push_back({TOKEN_LT, "<"});
                    pos++;
                    continue;
                }
                else if (c == '>')
                {
                    tokens.push_back({TOKEN_GT, ">"});
                    pos++;
                    continue;
                }
                else if (c == '(')
                {
                    tokens.push_back({TOKEN_LPAREN, "("});
                    pos++;
                    continue;
                }
                else if (c == ')')
                {
                    tokens.push_back({TOKEN_RPAREN, ")"});
                    pos++;
                    continue;
                }
                else if (c == '{')
                {
                    tokens.push_back({TOKEN_LBRACE, "{"});
                    pos++;
                    continue;
                }
                else if (c == '}')
                {
                    tokens.push_back({TOKEN_RBRACE, "}"});
                    pos++;
                    continue;
                }
                else if (c == '[')
                {
                    tokens.push_back({TOKEN_LBRACKET, "["});
                    pos++;
                    continue;
                }
                else if (c == ']')
                {
                    tokens.push_back({TOKEN_RBRACKET, "]"});
                    pos++;
                    continue;
                }
                else if (c == '.')
                {
                    tokens.push_back({TOKEN_DOT, "."});
                    pos++;
                    continue;
                }
                if (c == '=')
                {
                    tokens.push_back({TOKEN_ASSIGN, "="});
                    pos++;
                    continue;
                }
                else if (c == ',')
                {
                    tokens.push_back({TOKEN_COMMA, ","});
                    pos++;
                    continue;
                }

                tokens.push_back({TOKEN_OPERATOR, std::string(1, c)});
                pos++;
                continue;
            }
            pos++;
        }
        return tokens;
    }

private:
    std::string source;
    size_t pos;

    Token readNumber()
    {
        size_t start = pos;
        bool hasDot = false;
        while (pos < source.size() && (isdigit(source[pos]) || source[pos] == '.'))
        {
            if (source[pos] == '.')
                hasDot = true;
            pos++;
        }
        std::string num = source.substr(start, pos - start);
        return {hasDot ? TOKEN_FLOAT : TOKEN_INT, num};
    }

    Token readIdentifierOrKeyword()
    {
        size_t start = pos;
        while (pos < source.size() && (isalnum(source[pos]) || source[pos] == '_'))
        {
            pos++;
        }
        std::string id = source.substr(start, pos - start);

        if (id == "true" || id == "false")
            return {TOKEN_BOOL, id};
        if (id == "bikin")
            return {TOKEN_KEYWORD, id};
        if (id == "baka")
            return {TOKEN_KEYWORD, id};
        if (id == "kalo")
            return {TOKEN_IF, id};
        if (id == "laen")
            return {TOKEN_ELSE, id};
        if (id == "moshi")
            return {TOKEN_FUNC, id};
        if (id == "balik")
            return {TOKEN_RETURN, id};
        if (id == "bentar")
            return {TOKEN_WHILE, id};
        if (id == "lakuin")
            return {TOKEN_DO, id};

        return {TOKEN_IDENTIFIER, id};
    }

    Token readString()
    {
        char quote = source[pos++];
        size_t start = pos;
        while (pos < source.size() && source[pos] != quote)
        {
            pos++;
        }
        std::string str = source.substr(start, pos - start);
        pos++;
        return {TOKEN_STRING, str};
    }
};