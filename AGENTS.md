# 工作区约定

## 项目目录

克隆或创建新项目时，默认放在当前工作区目录的父目录下，即 `../<项目名>`，使其与当前项目目录平级。除非用户明确指定其他位置。

## 新项目创建前的开源复用检查

收到创建新项目的请求时，默认先根据用户提出的功能需求检索已有开源项目。若缺少足以检索的功能描述，先询问项目用途，不要立即创建。

如果发现核心功能、目标平台与约束高度匹配，且许可证可用、项目仍在维护的开源候选，必须先向用户说明候选及匹配理由，确认是复用、Fork、集成还是仍然新建；用户确认前不得创建项目。没有高匹配候选时可继续创建。无法完成检索时，应说明限制并确认是否继续。用户明确要求跳过检索时可以直接创建。

## 持久化偏好

需要在未来会话中保留的用户偏好、工作约定或项目规则，必须写入适当的 Markdown 指令文件（优先为本文件），不能只依赖当前对话上下文。

## 新功能实现流程

When implementing a new feature:

1. Define the expected behavior.
2. Write or update tests first.
3. Run tests and confirm:
    * existing tests pass
    * new tests fail for the expected reason
4. Implement the minimum code required.
5. Run the full relevant test suite and make it green.
6. Verify the actual feature behavior.
7. If behavior is wrong, fix the test/expectation first, then the implementation.
8. Refactor only after behavior and tests are correct.
9. For exploratory, research, feasibility-check, or PoC work, this workflow is not required. First validate the idea or technical approach; once it becomes production work, return to the workflow above.
