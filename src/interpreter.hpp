#pragma once

#include "token.hpp"
#include "lexer.hpp"

#include <iostream>
#include <vector>
#include <string>
#include <cctype>
#include <sstream>
#include <unordered_map>

struct Value
{
    WBType type = WB_NONE;
    std::string strValue;
    int intValue = 0;
    double floatValue = 0.0;
    bool boolValue = false;

    std::string toString() const
    {
        switch (type)
        {
        case WB_INT:
            return std::to_string(intValue);
        case WB_FLOAT:
            return std::to_string(floatValue);
        case WB_BOOL:
            return boolValue ? "true" : "false";
        case WB_STRING:
            return strValue;
        default:
            return "none";
        }
    }
};

struct ExecResult
{
    Value returnValue;
    bool hasReturn;
};

class Interpreter
{
public:
    ExecResult eval(const std::vector<Token> &tokens)
    {
        auto it = tokens.begin();
        while (it != tokens.end())
        {
            Token t = *it;

            if (t.type == TOKEN_KEYWORD && t.value == "bikin")
            {
                if (std::distance(it, tokens.end()) >= 4 && (it + 1)->type == TOKEN_IDENTIFIER && (it + 2)->value == "=")
                {
                    std::string varName = (it + 1)->value;
                    auto exprStart = it + 3;
                    auto exprResult = evalExpression(exprStart, tokens.end());
                    if (!exprResult.success)
                    {
                        ++it;
                        continue;
                    }
                    variables[varName] = exprResult.value;
                    it = exprStart;
                    continue;
                }
            }
            else if (t.type == TOKEN_KEYWORD && t.value == "baka")
            {
                if (std::distance(it, tokens.end()) >= 2)
                {
                    auto exprStart = it + 1;
                    auto exprResult = evalExpression(exprStart, tokens.end());
                    if (!exprResult.success)
                    {
                        ++it;
                        continue;
                    }
                    builtin_print(exprResult.value);
                    it = exprStart;
                    continue;
                }
            }
            else if (t.type == TOKEN_IF)
            {
                handleIf(it, tokens.end());
                continue;
            }
            else if (t.type == TOKEN_FUNC)
            {
                handleFunc(it, tokens.end());
                continue;
            }
            else if (t.type == TOKEN_WHILE)
            {
                handleWhile(it, tokens.end());
                continue;
            }
            else if (t.type == TOKEN_DO)
            {
                handleDoWhile(it, tokens.end());
                continue;
            }
            else
            {
                auto exprStart = it;
                auto exprResult = evalExpression(exprStart, tokens.end());
                if (!exprResult.success)
                {
                    ++it;
                }
                else
                {
                    it = exprStart;
                }
                continue;
            }

            ++it;
        }
        return {Value(), false};
    }

private:
    std::unordered_map<std::string, Value> variables;

    struct ParseResult
    {
        Value value;
        bool success;
    };

    struct Function
    {
        std::vector<std::string> params;
        std::vector<Token> body;
    };

    std::unordered_map<std::string, Function> functions;

    bool executeStatement(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        if (it == end)
            return false;

        Token t = *it;

        if (t.type == TOKEN_KEYWORD && t.value == "bikin")
        {
            if (std::distance(it, end) >= 4 && (it + 1)->type == TOKEN_IDENTIFIER && (it + 2)->value == "=")
            {
                std::string varName = (it + 1)->value;
                auto exprStart = it + 3;
                auto exprResult = evalExpression(exprStart, end);
                if (!exprResult.success)
                {
                    it += 3;
                    return false;
                }
                variables[varName] = exprResult.value;
                it = exprStart;
                return true;
            }
            else
            {
                std::cerr << "Error: statementnya invalid ganti lagi onechann!!" << std::endl;
                ++it;
                return false;
            }
        }

        if (t.type == TOKEN_IDENTIFIER)
        {
            auto nextIt = it + 1;
            if (nextIt != end && nextIt->type == TOKEN_ASSIGN)
            {
                std::string varName = t.value;
                auto exprStart = nextIt + 1;
                auto exprResult = evalExpression(exprStart, end);
                if (!exprResult.success)
                {
                    it = exprStart;
                    return false;
                }

                if (variables.count(varName) == 0)
                {
                    std::cerr << "Error: variable '" << varName << "' itu gak ada coba cari kalo ada ￣へ￣" << std::endl;
                    it = exprStart;
                    return false;
                }

                variables[varName] = exprResult.value;
                it = exprStart;
                return true;
            }
        }

        auto exprStart = it;
        auto exprResult = evalExpression(exprStart, end);
        if (exprResult.success)
        {
            it = exprStart;
            return true;
        }

        std::cerr << "Error: '" << t.value << "' <- ini apaan oneechan（︶^︶）" << std::endl;
        ++it;
        return false;
    }

    ParseResult evalExpression(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        auto result = parseOr(it, end);
        return result;
    }

    ExecResult executeBlockWithReturn(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        ExecResult result;
        result.hasReturn = false;

        while (it != end && it->type != TOKEN_RBRACE)
        {
            Token t = *it;

            if (t.type == TOKEN_KEYWORD && t.value == "bikin")
            {
                if (std::distance(it, end) >= 4 && (it + 1)->type == TOKEN_IDENTIFIER && (it + 2)->value == "=")
                {
                    std::string varName = (it + 1)->value;
                    auto exprStart = it + 3;
                    auto exprResult = evalExpression(exprStart, end);
                    if (exprResult.success)
                    {
                        variables[varName] = exprResult.value;
                        it = exprStart;
                        continue;
                    }
                    else
                    {
                        it += 3;
                        continue;
                    }
                }
            }

            if (t.type == TOKEN_KEYWORD && t.value == "baka")
            {
                if (std::distance(it, end) >= 1)
                {
                    auto exprStart = it + 1;
                    auto exprResult = evalExpression(exprStart, end);
                    if (exprResult.success)
                    {
                        builtin_print(exprResult.value);
                        it = exprStart;
                        continue;
                    }
                    else
                    {
                        ++it;
                        continue;
                    }
                }
            }

            if (t.type == TOKEN_IF)
            {
                auto ifResult = handleIf(it, end);
                if (ifResult.hasReturn)
                {
                    return ifResult;
                }
                continue;
            }

            if (t.type == TOKEN_RETURN)
            {
                auto exprStart = it + 1;
                auto exprResult = evalExpression(exprStart, end);
                if (exprResult.success)
                {
                    result.returnValue = exprResult.value;
                    result.hasReturn = true;
                    it = exprStart;
                    return result;
                }
                else
                {
                    ++it;
                    continue;
                }
            }

            if (t.type == TOKEN_FUNC)
            {
                handleFunc(it, end);
                continue;
            }

            if (t.type == TOKEN_WHILE)
            {
                handleWhile(it, end);
                continue;
            }

            if (t.type == TOKEN_DO)
            {
                handleDoWhile(it, end);
                continue;
            }

            if (t.type == TOKEN_IDENTIFIER)
            {
                auto nextIt = it + 1;
                if (nextIt != end && nextIt->type == TOKEN_ASSIGN)
                {
                    std::string varName = t.value;
                    auto exprStart = nextIt + 1;
                    auto exprResult = evalExpression(exprStart, end);
                    if (!exprResult.success)
                    {
                        it = exprStart;
                        continue;
                    }

                    if (variables.count(varName) == 0)
                    {
                        std::cerr << "Error: oneechan variable '" << varName << "' itu gak ada!!" << std::endl;
                        it = exprStart;
                        continue;
                    }

                    variables[varName] = exprResult.value;
                    it = exprStart;
                    continue;
                }
            }

            auto exprStart = it;
            auto exprResult = evalExpression(exprStart, end);
            if (!exprResult.success)
            {
                ++it;
            }
            else
            {
                it = exprStart;
            }
            continue;
        }

        if (it != end && it->type == TOKEN_RBRACE)
            ++it;
        return result;
    }

    void handleFunc(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        ++it;
        if (it == end || it->type != TOKEN_IDENTIFIER)
        {
            std::cerr << "Error: gak tau apa errornya!! ￣へ￣" << std::endl;
            return;
        }

        std::string funcName = it->value;
        ++it;

        if (it == end || it->type != TOKEN_LPAREN)
        {
            std::cerr << "Error: error karena ada sesuatu yang belum dikasih '(' dasar begoo!! " << std::endl;
            return;
        }
        ++it;

        std::vector<std::string> params;
        while (it != end && it->type != TOKEN_RPAREN)
        {
            if (it->type == TOKEN_IDENTIFIER)
            {
                params.push_back(it->value);
                ++it;
                if (it != end && it->type == TOKEN_COMMA)
                {
                    ++it;
                    continue;
                }
                else if (it->type == TOKEN_RPAREN)
                {
                    break;
                }
                else
                {
                    std::cerr << "Error: sepertinnya suatu fungsi ada yang belum dikasih ',' atau '()' (´。＿。｀)" << std::endl;
                    return;
                }
            }
            else
            {
                std::cerr << "Error: suatu nama parameter enggak valid (￣﹏￣；)" << std::endl;
                return;
            }
        }

        if (it == end || it->type != TOKEN_RPAREN)
        {
            std::cerr << "Error: kurang ')' bego!!╰（‵□′）╯" << std::endl;
            return;
        }
        ++it;

        if (it == end || it->type != TOKEN_LBRACE)
        {
            std::cerr << "Error: ada suatu fungsi kurang '{' bego banget (￣﹏￣；)" << std::endl;
            return;
        }
        ++it;

        std::vector<Token> body;
        int depth = 1;
        while (it != end)
        {
            if (it->type == TOKEN_LBRACE)
                depth++;
            else if (it->type == TOKEN_RBRACE)
            {
                depth--;
                if (depth == 0)
                {
                    ++it;
                    break;
                }
            }
            body.push_back(*it);
            ++it;
        }

        functions[funcName] = {params, body};
    }

    ParseResult callFunction(const std::string &funcName, std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        if (funcName == "exec")
        {
            if (it == end || it->type != TOKEN_LPAREN)
            {
                std::cerr << "Error: kurang '(' buat panggil exec (￣﹏￣；)" << std::endl;
                return {Value(), false};
            }
            ++it;

            auto exprResult = evalExpression(it, end);
            if (!exprResult.success)
            {
                return {Value(), false};
            }

            if (it == end || it->type != TOKEN_RPAREN)
            {
                std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
                return {Value(), false};
            }
            ++it;

            if (exprResult.value.type != WB_STRING)
            {
                std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
                return {Value(), false};
            }

            int result = std::system(exprResult.value.strValue.c_str());
            Value retVal;
            retVal.type = WB_INT;
            retVal.intValue = result;
            return {retVal, true};
        }

        if (funcName == "input")
        {
            if (it == end || it->type != TOKEN_LPAREN)
            {
                std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
                return {Value(), false};
            }
            ++it;

            auto exprResult = evalExpression(it, end);
            if (!exprResult.success)
            {
                return {Value(), false};
            }

            if (it == end || it->type != TOKEN_RPAREN)
            {
                std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
                return {Value(), false};
            }
            ++it;

            if (exprResult.value.type != WB_STRING)
            {
                std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
                return {Value(), false};
            }

            std::cout << exprResult.value.strValue;
            std::string userInput;
            std::getline(std::cin, userInput);

            Value retVal;

            try
            {
                size_t pos = 0;
                int intValue = std::stoi(userInput, &pos);
                if (pos == userInput.length())
                {
                    retVal.type = WB_INT;
                    retVal.intValue = intValue;
                    return {retVal, true};
                }
            }
            catch (...)
            {
            }

            try
            {
                size_t pos = 0;
                double floatValue = std::stod(userInput, &pos);
                if (pos == userInput.length())
                {
                    retVal.type = WB_FLOAT;
                    retVal.floatValue = floatValue;
                    if (floatValue == static_cast<int>(floatValue))
                    {
                        retVal.type = WB_INT;
                        retVal.intValue = static_cast<int>(floatValue);
                    }
                    return {retVal, true};
                }
            }
            catch (...)
            {
            }

            retVal.type = WB_STRING;
            retVal.strValue = userInput;
            return {retVal, true};
        }

        if (functions.find(funcName) == functions.end())
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return {Value(), false};
        }

        const Function &func = functions[funcName];

        if (it == end || it->type != TOKEN_LPAREN)
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return {Value(), false};
        }
        ++it;

        std::vector<Value> args;
        while (it != end && it->type != TOKEN_RPAREN)
        {
            auto exprStart = it;
            auto exprResult = evalExpression(exprStart, end);
            if (!exprResult.success)
            {
                return {Value(), false};
            }
            args.push_back(exprResult.value);
            it = exprStart;

            if (it != end && it->type == TOKEN_COMMA)
            {
                ++it;
            }
            else if (it->type != TOKEN_RPAREN)
            {
                std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
                return {Value(), false};
            }
        }

        if (it == end || it->type != TOKEN_RPAREN)
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return {Value(), false};
        }
        ++it;

        if (args.size() != func.params.size())
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return {Value(), false};
        }

        auto oldVars = variables;

        for (size_t i = 0; i < func.params.size(); ++i)
        {
            variables[func.params[i]] = args[i];
        }

        auto bodyIt = func.body.begin();
        auto execResult = executeBlockWithReturn(bodyIt, func.body.end());

        variables = oldVars;

        if (execResult.hasReturn)
        {
            return {execResult.returnValue, true};
        }
        else
        {
            return {Value(), true};
        }
    }

    ParseResult parseOr(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        auto left = parseAnd(it, end);
        if (!left.success)
            return left;

        while (it != end && it->type == TOKEN_OR)
        {
            ++it;
            auto right = parseAnd(it, end);
            if (!right.success)
                return {Value(), false};

            Value result;
            result.type = WB_BOOL;
            result.boolValue = toBool(left.value) || toBool(right.value);
            left.value = result;
        }
        return left;
    }

    ParseResult parseAnd(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        auto left = parseComparison(it, end);
        if (!left.success)
            return left;

        while (it != end && it->type == TOKEN_AND)
        {
            ++it;
            auto right = parseComparison(it, end);
            if (!right.success)
                return {Value(), false};

            Value result;
            result.type = WB_BOOL;
            result.boolValue = toBool(left.value) && toBool(right.value);
            left.value = result;
        }
        return left;
    }

    ParseResult parseComparison(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        auto left = parseAdditive(it, end);
        if (!left.success)
            return left;

        while (it != end)
        {
            TokenType op = it->type;
            if (op != TOKEN_EQ && op != TOKEN_NEQ && op != TOKEN_LT && op != TOKEN_GT &&
                op != TOKEN_LE && op != TOKEN_GE)
            {
                break;
            }
            ++it;

            auto right = parseAdditive(it, end);
            if (!right.success)
                return {Value(), false};

            Value result;
            result.type = WB_BOOL;

            if (op == TOKEN_EQ)
            {
                result.boolValue = isEqual(left.value, right.value);
            }
            else if (op == TOKEN_NEQ)
            {
                result.boolValue = !isEqual(left.value, right.value);
            }
            else
            {
                double leftNum = toNumber(left.value);
                double rightNum = toNumber(right.value);

                if (op == TOKEN_LT)
                    result.boolValue = leftNum < rightNum;
                else if (op == TOKEN_GT)
                    result.boolValue = leftNum > rightNum;
                else if (op == TOKEN_LE)
                    result.boolValue = leftNum <= rightNum;
                else if (op == TOKEN_GE)
                    result.boolValue = leftNum >= rightNum;
            }

            left.value = result;
        }
        return left;
    }

    ParseResult parseAdditive(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        auto left = parseMultiplicative(it, end);
        if (!left.success)
            return left;

        while (it != end && (it->type == TOKEN_PLUS || it->type == TOKEN_MINUS))
        {
            TokenType op = it->type;
            ++it;

            auto right = parseMultiplicative(it, end);
            if (!right.success)
                return {Value(), false};

            Value result;
            if (left.value.type == WB_STRING || right.value.type == WB_STRING)
            {
                result.type = WB_STRING;
                result.strValue = left.value.toString() + right.value.toString();
            }
            else
            {
                double leftNum = toNumber(left.value);
                double rightNum = toNumber(right.value);
                result.type = WB_FLOAT;
                if (op == TOKEN_PLUS)
                {
                    result.floatValue = leftNum + rightNum;
                }
                else
                {
                    result.floatValue = leftNum - rightNum;
                }
                if (result.floatValue == static_cast<int>(result.floatValue))
                {
                    result.type = WB_INT;
                    result.intValue = static_cast<int>(result.floatValue);
                }
            }
            left.value = result;
        }
        return left;
    }

    ParseResult parseMultiplicative(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        auto left = parsePostfix(it, end);
        if (!left.success)
            return left;

        while (it != end && (it->type == TOKEN_STAR || it->type == TOKEN_SLASH))
        {
            TokenType op = it->type;
            ++it;

            auto right = parsePostfix(it, end);
            if (!right.success)
                return {Value(), false};

            double leftNum = toNumber(left.value);
            double rightNum = toNumber(right.value);

            Value result;
            result.type = WB_FLOAT;
            if (op == TOKEN_STAR)
            {
                result.floatValue = leftNum * rightNum;
            }
            else
            {
                if (rightNum == 0)
                {
                    std::cerr << "Error: division by zero" << std::endl;
                    return {Value(), false};
                }
                result.floatValue = leftNum / rightNum;
            }

            if (result.floatValue == static_cast<int>(result.floatValue))
            {
                result.type = WB_INT;
                result.intValue = static_cast<int>(result.floatValue);
            }

            left.value = result;
        }
        return left;
    }

    ParseResult parsePrimary(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        if (it == end)
            return {Value(), false};

        if (it->type == TOKEN_MINUS)
        {
            ++it;
            auto operand = parsePrimary(it, end);
            if (!operand.success)
                return {Value(), false};

            Value result;
            double num = toNumber(operand.value);
            result.type = WB_FLOAT;
            result.floatValue = -num;

            if (result.floatValue == static_cast<int>(result.floatValue))
            {
                result.type = WB_INT;
                result.intValue = static_cast<int>(result.floatValue);
            }

            return {result, true};
        }

        if (it->type == TOKEN_LPAREN)
        {
            ++it;
            auto result = evalExpression(it, end);
            if (!result.success)
                return {Value(), false};

            if (it == end || it->type != TOKEN_RPAREN)
            {
                std::cerr << "Error: expected ')'" << std::endl;
                return {Value(), false};
            }
            ++it;
            return result;
        }

        if (it->type == TOKEN_IDENTIFIER)
        {
            auto funcIt = it;
            std::string funcName = it->value;
            ++funcIt;

            if (funcIt != end && funcIt->type == TOKEN_LPAREN)
            {
                it = funcIt;
                return callFunction(funcName, it, end);
            }
        }

        Value val = tokenToValue(*it);
        ++it;

        if (val.type == WB_NONE)
        {
            return {Value(), false};
        }

        return {val, true};
    }

    bool toBool(const Value &v)
    {
        switch (v.type)
        {
        case WB_BOOL:
            return v.boolValue;
        case WB_INT:
            return v.intValue != 0;
        case WB_FLOAT:
            return v.floatValue != 0.0;
        case WB_STRING:
            return !v.strValue.empty();
        // 🚫 case WB_ARRAY DIHAPUS
        default:
            return false;
        }
    }

    double toNumber(const Value &v)
    {
        switch (v.type)
        {
        case WB_INT:
            return static_cast<double>(v.intValue);
        case WB_FLOAT:
            return v.floatValue;
        case WB_BOOL:
            return v.boolValue ? 1.0 : 0.0;
        case WB_STRING:
        {
            try
            {
                return std::stod(v.strValue);
            }
            catch (...)
            {
                return 0.0;
            }
        }
        default:
            return 0.0;
        }
    }

    bool isEqual(const Value &a, const Value &b)
    {
        if (a.type != b.type)
            return false;
        switch (a.type)
        {
        case WB_INT:
            return a.intValue == b.intValue;
        case WB_FLOAT:
            return a.floatValue == b.floatValue;
        case WB_BOOL:
            return a.boolValue == b.boolValue;
        case WB_STRING:
            return a.strValue == b.strValue;
        default:
            return false;
        }
    }

    Value tokenToValue(const Token &t)
    {
        Value v;

        switch (t.type)
        {
        case TOKEN_INT:
            v.type = WB_INT;
            v.intValue = std::stoi(t.value);
            break;
        case TOKEN_FLOAT:
            v.type = WB_FLOAT;
            v.floatValue = std::stod(t.value);
            break;
        case TOKEN_BOOL:
            v.type = WB_BOOL;
            v.boolValue = (t.value == "true");
            break;
        case TOKEN_STRING:
            v.type = WB_STRING;
            v.strValue = t.value;
            break;
        case TOKEN_IDENTIFIER:
            if (variables.count(t.value))
                return variables[t.value];
            else
            {
                std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
                v.type = WB_NONE;
                return v;
            }
            break;
        default:
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << t.type << std::endl;
            v.type = WB_NONE;
            return v;
        }
        return v;
    }

    bool evaluateCondition(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        auto result = evalExpression(it, end);
        if (!result.success)
            return false;
        return toBool(result.value);
    }

    void executeBlock(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        while (it != end && it->type != TOKEN_RBRACE)
        {
            Token t = *it;

            if (t.type == TOKEN_KEYWORD && t.value == "bikin")
            {
                if (std::distance(it, end) >= 4 && (it + 1)->type == TOKEN_IDENTIFIER && (it + 2)->value == "=")
                {
                    std::string varName = (it + 1)->value;
                    auto exprStart = it + 3;
                    auto exprResult = evalExpression(exprStart, end);
                    if (exprResult.success)
                    {
                        variables[varName] = exprResult.value;
                        it = exprStart;
                        continue;
                    }
                    else
                    {
                        it += 3;
                        continue;
                    }
                }
            }

            if (t.type == TOKEN_KEYWORD && t.value == "baka")
            {
                if (std::distance(it, end) >= 1)
                {
                    auto exprStart = it + 1;
                    auto exprResult = evalExpression(exprStart, end);
                    if (exprResult.success)
                    {
                        builtin_print(exprResult.value);
                        it = exprStart;
                        continue;
                    }
                    else
                    {
                        ++it;
                        continue;
                    }
                }
            }

            if (t.type == TOKEN_IF)
            {
                handleIf(it, end);
                continue;
            }

            ++it;
        }
        if (it != end && it->type == TOKEN_RBRACE)
            ++it;
    }

    ExecResult handleIf(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        ExecResult result;
        result.hasReturn = false;

        ++it;
        if (it == end)
            return result;

        auto condIt = it;
        bool conditionMet = evaluateCondition(condIt, end);
        it = condIt;

        if (it == end || it->type != TOKEN_LBRACE)
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return result;
        }
        ++it;

        if (conditionMet)
        {
            auto execResult = executeBlockWithReturn(it, end);
            if (execResult.hasReturn)
            {
                return execResult;
            }
        }
        else
        {
            int depth = 1;
            while (it != end)
            {
                if (it->type == TOKEN_LBRACE)
                    depth++;
                else if (it->type == TOKEN_RBRACE)
                {
                    depth--;
                    if (depth == 0)
                    {
                        ++it;
                        break;
                    }
                }
                ++it;
            }
        }

        while (it != end && it->type == TOKEN_ELSE)
        {
            ++it;
            if (it != end && it->type == TOKEN_IF)
            {
                ++it;
                auto elseIfIt = it;
                bool elseIfCond = evaluateCondition(elseIfIt, end);
                it = elseIfIt;

                if (it == end || it->type != TOKEN_LBRACE)
                    break;
                ++it;

                if (!conditionMet && elseIfCond)
                {
                    auto execResult = executeBlockWithReturn(it, end);
                    if (execResult.hasReturn)
                    {
                        return execResult;
                    }
                }
                else
                {
                    int depth = 1;
                    while (it != end)
                    {
                        if (it->type == TOKEN_LBRACE)
                            depth++;
                        else if (it->type == TOKEN_RBRACE)
                        {
                            depth--;
                            if (depth == 0)
                            {
                                ++it;
                                break;
                            }
                        }
                        ++it;
                    }
                }
            }
            else if (it != end && it->type == TOKEN_LBRACE)
            {
                ++it;
                if (!conditionMet)
                {
                    auto execResult = executeBlockWithReturn(it, end);
                    if (execResult.hasReturn)
                    {
                        return execResult;
                    }
                }
                else
                {
                    int depth = 1;
                    while (it != end)
                    {
                        if (it->type == TOKEN_LBRACE)
                            depth++;
                        else if (it->type == TOKEN_RBRACE)
                        {
                            depth--;
                            if (depth == 0)
                            {
                                ++it;
                                break;
                            }
                        }
                        ++it;
                    }
                }
                break;
            }
            else
            {
                break;
            }
        }

        return result;
    }

    void handleWhile(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        ++it;

        if (it == end || it->type != TOKEN_LPAREN)
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return;
        }
        ++it;

        auto condStart = it;
        auto bodyStart = it;

        int parenDepth = 1;
        while (bodyStart != end && parenDepth > 0)
        {
            if (bodyStart->type == TOKEN_LPAREN)
                parenDepth++;
            else if (bodyStart->type == TOKEN_RPAREN)
                parenDepth--;
            if (parenDepth > 0)
                ++bodyStart;
        }

        if (bodyStart == end || bodyStart->type != TOKEN_RPAREN)
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return;
        }

        auto condEnd = bodyStart;
        ++bodyStart;

        if (bodyStart == end || bodyStart->type != TOKEN_LBRACE)
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return;
        }
        ++bodyStart;

        std::vector<Token> bodyTokens;
        int braceDepth = 1;
        auto bodyIt = bodyStart;
        while (bodyIt != end && braceDepth > 0)
        {
            if (bodyIt->type == TOKEN_LBRACE)
                braceDepth++;
            else if (bodyIt->type == TOKEN_RBRACE)
                braceDepth--;
            if (braceDepth > 0)
                bodyTokens.push_back(*bodyIt);
            ++bodyIt;
        }

        if (braceDepth != 0)
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return;
        }

        auto condIt = condStart;
        auto condResult = evaluateCondition(condIt, condEnd);
        while (condResult)
        {
            auto localIt = bodyTokens.cbegin();
            auto execResult = executeBlockWithReturn(localIt, bodyTokens.cend());

            if (execResult.hasReturn)
            {
                it = bodyIt;
                return;
            }

            condIt = condStart;
            condResult = evaluateCondition(condIt, condEnd);
        }

        it = bodyIt;
    }

    void handleDoWhile(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        ++it;

        if (it == end || it->type != TOKEN_LBRACE)
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return;
        }
        ++it;

        std::vector<Token> bodyTokens;
        int braceDepth = 1;
        while (it != end && braceDepth > 0)
        {
            if (it->type == TOKEN_LBRACE)
                braceDepth++;
            else if (it->type == TOKEN_RBRACE)
                braceDepth--;
            if (braceDepth > 0)
                bodyTokens.push_back(*it);
            ++it;
        }

        if (braceDepth != 0)
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return;
        }

        if (it == end || it->type != TOKEN_WHILE)
        {
            std::cerr << "Error: gak tau apa errornya jelasin sendiri :p" << std::endl;
            return;
        }
        ++it;

        if (it == end || it->type != TOKEN_LPAREN)
        {
            std::cerr << "Error: kurang '(' begoo!!" << std::endl;
            return;
        }
        ++it;

        auto condStart = it;
        int parenDepth = 1;
        while (it != end && parenDepth > 0)
        {
            if (it->type == TOKEN_LPAREN)
                parenDepth++;
            else if (it->type == TOKEN_RPAREN)
                parenDepth--;
            if (parenDepth > 0)
                ++it;
        }

        if (it == end || it->type != TOKEN_RPAREN)
        {
            std::cerr << "Error: ada yang kurang ')' bego!!" << std::endl;
            return;
        }
        auto condEnd = it;
        ++it;

        bool condResult = true;
        do
        {
            auto localIt = bodyTokens.cbegin();
            auto execResult = executeBlockWithReturn(localIt, bodyTokens.cend());

            if (execResult.hasReturn)
            {
                return;
            }

            auto tempIt = condStart;
            condResult = evaluateCondition(tempIt, condEnd);
        } while (condResult);
    }

    ParseResult parsePostfix(std::vector<Token>::const_iterator &it, std::vector<Token>::const_iterator end)
    {
        auto left = parsePrimary(it, end);
        if (!left.success)
            return left;

        while (it != end)
        {
            break;
        }

        return left;
    }

    void builtin_print(const Value &v)
    {
        std::cout << v.toString() << std::endl;
    }
};