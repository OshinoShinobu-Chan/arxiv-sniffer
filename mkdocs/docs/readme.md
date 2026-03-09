# 网站说明

这个网站是由[ArXiv-Sniffer](https://github.com/OshinoShinobu-Chan/arxiv-sniffer)项目自动构建的。本页面是对这个网站内容的说明。本页面的位置在`mkdocs/docs/readme.md`。

## 网站首页

网站首页在`mkdocs/docs/index.md`。网站的首页默认是空的。你可以根据你的需要在你fork的项目中装扮它。(放一些你自己的logo什么的)<del>绝对不是我懒得做一个fancy的网站首页哦。绝对不是</del>

## 上方导航栏

上方的导航栏根据你订阅的topics对页面进行分类。点击对应的项目之后应该能跳转到相应的topics**最新日期**的页面。

除了对应于这个说明页面的项，其他的导航项都是根据你的配置文件自动生成的。

## 网站结构

网站下根据主题，在`mkdocs/docs`下分成多个文件夹。这些文件夹也是自动生成的，文件夹的名称是你在中设置你订阅的主题的名称。每个文件下下面有一个日期命名的markdown文件，这个文件就是每日从arXiv上爬取和筛选得到的论文的具体总结页面。

## 总结页面

每个主题每天的总结页面是根据`mkdocs/docs/templates`下面的模板文件生成的。在这些模板中类似`{...}`的一些匹配项会被替换成对应的内容。下面介绍支持的匹配项，你根据需要编写你自己喜欢的模板。

### page_template.md

- `{date}`: 页面对应的日期，格式为YYYY-MM-DD(例如：2026-03-06)。
- `{topic}`：页面对应的主题的描述。
- `{papers}`：每篇论文的总结根据`paper_template.md`生成之后使用空行分隔形成的字符串。
- `{yesterday_link}`：前一天的论文总结页面的路径。总是会尝试拼接形成一个相对路径。如果前一天的页面不存在的话将会是一个无效的路径。
- `{tomorrow_link}`：后一天的论文总结页面的路径。同样，如果后一天的页面不存在的话，将会是一个无效的路径。

### paper_template.md

- `{topic_relevance}`：当前论文与主题的相关程度评分。这个项目由`topic_relevance_template.md`和`dimension_template.md`生成。
- `{AI_name}`：使用的AI模型的名称。在配置文件中设置。
- `{abstract}`：论文的摘要原文。
- `{arXiv_link}`：arXiv上的论文链接。
- `{pdf_link}`：arXiv上论文pdf的链接。使用markdown超链接的格式呈现可以实现直接点击下载对应pdf。
- `{src_link}`：arXiv上论文Tex格式压缩包的链接。使用markdown超链接的格式呈现可以实现直接点击下载对应LaTeX原文。

### topic_relevance_template.md

- `{dimension_details}`: 相关性评价的量表中每个维度对应的内容使用`dimension_template.md`生成之后，使用空行分隔拼接形成的字符串。
- `{overall_score}`：量表中的分数经过加权平均之后换算到100分满分的分数。
- `{comment_text}`：在源代码中写死的一个分数对应的评价，目前不支持修改。

### dimension_template.md

- `{name}`：当前dimension的名称。在`/prompts/relevance_dimension.json`中配置。
- `{description}`：当前dimension具体含义的描述。在`/prompts/relevance_dimension.json`中配置。
- `{weight}`：当前dimension的权重。在`/prompts/relevance_dimension.json`中配置。
- `{score}`：当前dimension的得分。由LLM生成。
- `{reason}`：当前dimension打分的理由。由LLM生成。