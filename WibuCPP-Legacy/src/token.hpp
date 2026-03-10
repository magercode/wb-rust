#pragma once
#include <string>

enum WBType
{
    WB_NONE,
    WB_INT,
    WB_FLOAT,
    WB_BOOL,
    WB_STRING,
    // WB_ARRAY,
};

enum TokenType
{
    TOKEN_NONE,
    TOKEN_INT,
    TOKEN_FLOAT,
    TOKEN_BOOL,
    TOKEN_STRING,
    TOKEN_IDENTIFIER,
    TOKEN_OPERATOR,
    TOKEN_PUNCTUATION,
    TOKEN_KEYWORD,
    TOKEN_IF,
    TOKEN_ELSE,
    TOKEN_LBRACE,
    TOKEN_RBRACE,
    TOKEN_LBRACKET, // [
    TOKEN_RBRACKET, // ]
    TOKEN_ASSIGN,  // =
    TOKEN_PLUS,   // +
    TOKEN_MINUS,  // -
    TOKEN_STAR,   // *
    TOKEN_SLASH,  // /
    TOKEN_EQ,     // ==
    TOKEN_NEQ,    // !=
    TOKEN_LT,     // <
    TOKEN_GT,     // >
    TOKEN_LE,     // <=
    TOKEN_GE,     // >=
    TOKEN_AND,    // &&
    TOKEN_OR,     // ||
    TOKEN_LPAREN, // (
    TOKEN_RPAREN, // )
    TOKEN_FUNC,   // func
    TOKEN_RETURN, // return
    TOKEN_COMMA,   // ,
    TOKEN_DOT,   // .
    TOKEN_WHILE,
    TOKEN_DO,
};


struct Token
{
    TokenType type;
    std::string value;
};