# ArXiv Sniffer

> Note: This README is translated from chinese version, totally by DeepSeek.

[中文](./README.md)

## Introduction

This project is a program based on large language models (LLMs) designed to filter and summarize papers from the arXiv catchup page (the daily subscription page).

## What This Project Can Do

Each time the core program runs, it performs the following process:

1.  Crawls the content of yesterday's catchup page from arXiv and extracts the entries for newly submitted papers.
2.  Users can subscribe to topics in advance (by describing the topics they are interested in using natural language in the configuration file). The program calls the LLM API to screen all the papers extracted in the previous step, selecting those highly relevant to the topics of interest. This screening is performed separately for each topic.
3.  After screening, the project automatically generates a display page for each topic using templates, showcasing the latest papers from the previous day.

If you deploy using the GitHub Action method mentioned below, the project will run automatically every day and deploy the built pages to GitHub Pages. This way, you can access the latest content daily just by visiting a single URL. You can also easily share the results with others (like collaborators, supervisors, etc.).

Details about the features in the process above will be introduced below.

## Why This Project Exists

The world has long suffered from arXiv's search function! (Or maybe it's just me). Anyway, out of dissatisfaction with arXiv's simple keyword-matching search, this project was developed. However, please note that this project is **not** a search engine for arXiv. It does not search historical articles; it only filters the latest daily articles.

This project is suitable if you are interested in a topic and want to start discovering papers in that field.

## Features of This Project (How is This Project Implemented?)

> Users who don't care about details and want to get started quickly, please skip this section.

### Crawling arXiv

This project uses the `reqwest` library to crawl arXiv. Because the URI structure of the arXiv website is clear, and the HTML structure of the pages is also clear, this process is simple. Once you have the arXiv ID of a paper, you can easily construct the URLs for its details page, PDF download, and TeX source download.

The only difficulty is that some paper abstracts are in MathJax format. However, I took the easy way out and didn't process MathJax carefully. This may result in poorly formatted extracted abstracts for some papers.

### LLM Relevance Evaluation

This part uses a multi-dimensional scale approach, asking the LLM to score the relevance of a paper to a given topic from multiple perspectives. A weighted average then produces a final score. For papers that are ultimately selected, the scores given by the LLM for each dimension, along with the reasons for those scores, are displayed on the page.

The scale I designed here was actually created by DeepSeek's deep thinking mode, with minor adjustments by me. See details [here](./prompts/relevance_dimensions.json).

With this scale, it is filled into the prompt [template](./prompts/relevance_template.txt) along with other important information and sent to the LLM. Finally, the `json` result returned by the LLM is parsed, and screening is performed based on a set threshold.

Currently, this project only supports DeepSeek. <del>Because I really like DeepSeek.</del>

> About token usage: You can make a very rough estimate (for reference only), about 1000 tokens per paper per topic. For example, if you subscribe to 3 topics in the cs field, and there are about 500 new papers in the cs field daily, you can estimate a daily consumption of $3 \times 500 \times 1000 = 1.5$ million tokens.

### Generating Display Pages

I used `mkdocs` to build the display pages and defaulted to the `material` theme for the page style. For details about the display website, please check [here](./mkdocs/docs/readme.md). In summary, pages are automatically generated based on a series of templates.

## How to Get Started Quickly

This project supports setting up scheduled tasks using GitHub Actions and automatically deploying to GitHub Pages. For first-time use, please follow the steps below.

### 0. Fork This Project

Before anything else, you need to copy this project to your own GitHub account. If you don't have a GitHub account, ask an AI to help you create one quickly.

Once you have your own GitHub account, you can click the `Fork` button in the top right corner to copy this project. See the image below.

![Fork button illustration](./readme_pic/fork_button.png)

You should enter a page like the one below. Just click "Create Fork" in the bottom right corner.

![Fork page illustration](./readme_pic/fork_page.png)

Then, navigate to the project copied under your own account to perform the following operations.

### 1. Configuration File

This project uses a `toml` format configuration file. You don't need to understand `toml`; the project provides comprehensive examples—just follow them. I will only introduce the necessary configuration items here. For other items not mentioned, keep the defaults to get started quickly. You can find the configuration file [here](./config.toml).

- **subject_code**: Line 4, change `"cs"` in the quotes to the arXiv code for your field.

    You can select your field on the arXiv website as shown below, then click search. Then check the browser URL; it should look like `https://arxiv.org/search/physics`. The part after the last `/` is your field's code.
    ![arXiv search illustration](./readme_pic/arXiv_search.png)

- **topics**: This configuration item is slightly more complex. Please follow the example in lines 14-20 of the default configuration file.

    You can see there are two lines with `[[topics]]` alone, each followed by two lines: `name = xxx` and `description = xxx`.

    Each `[[topics]]` group represents a subscribed topic.

    The `name` under each `[[topics]]` is the name of this topic. This name will be displayed in the navigation bar of the automatically built website. Please choose a short name. It can include Chinese, but it's best to avoid strange characters.

    The `description` below is where you can use a short paragraph to describe what you are interested in regarding this topic. The LLM will use this description to judge whether a paper is relevant to your topic. Be sure to enclose the content here in `"`, but do not use `"` within the content itself.

    Place as many groups here as the number of topics you want to subscribe to. The default configuration file subscribes to two topics.

If you are using GitHub for the first time, it's recommended to modify the configuration file directly on the webpage. Click on `config.toml` in your forked project page. You will enter an interface like below. Click the pencil icon in the top right corner to enter edit mode.

![GitHub edit interface illustration](./readme_pic/github_edit.png)

You can modify the file content directly in the edit interface. Then click the `Commit changes...` button in the top right corner. A window like the one below will pop up. Fill in a brief description in the `Commit message`, and then click `Commit changes` to successfully save your changes.

![Commit Changes illustration](./readme_pic/commit_changes.png)

### 2. Configure DeepSeek API Key

For the process of creating a DeepSeek account, recharging, and obtaining a DeepSeek API key, please ask DeepSeek yourself.

> Note: Do not put your key online or let others know your key.

Once you have a DeepSeek API key, you can add it to the project through the following steps.

First, click the `Settings` button near the top right of your forked project page, as shown below.

![GitHub Settings button illustration](./readme_pic/github_settings.png)

Then, on the left side of the page you enter, find the item highlighted in the red box in the image below and click it.

![GitHub secrets illustration](./readme_pic/github_secrets.png)

On the following page, click the `New repository secret` button.

![GitHub secrets interface illustration](./readme_pic/manage_secrets.png)

Then, in the interface below, enter "DEEPSEEK_API_KEY" in the `Name` field (must match exactly). Paste your DeepSeek API key into the `Secret` field. Finally, click `Add secret`.

![new secret interface illustration](./readme_pic/new_secret.png)

### 3. Configure GitHub Action

First, click the `Actions` button near the top middle of your forked project page. You should then see an interface like the one below. Click the green button in the middle.

![Actions interface illustration](./readme_pic/workflow.png)

After that, a workflow should appear on the left side of your page, but it should be `Disabled`. You need to click this workflow and then select `Enable workflow` to activate it. This task will then run automatically on a daily schedule.

![workflow interface illustration](./readme_pic/enable_workflow.png)

### 4. Create site Branch

Finally, we need a bit of configuration to automatically deploy the daily crawled and filtered results to GitHub Pages. There's a prerequisite step before this. Go back to the `Code` interface, and click on the area shown in the image below.

![branch illustration](./readme_pic/branch.png)

Then click `New branch` in the top right corner to enter the branch creation interface. As shown below, enter `site` (must match exactly) in the `New branch name` field, then click `Create new branch`.

![new branch illustration](./readme_pic/new_branch.png)

### 5. Configure GitHub Page

This is the final step: configure GitHub Pages to automatically deploy the `site` branch. First, click `Settings` at the top of the interface, then click `Pages` on the left (relevant illustrations are above, so I won't repeat them). You will then enter an interface similar to the one below. Click the dropdown menu in the `Branch` section in the middle, select `site`, then click `save`. That's it.

![GitHub Pages illustration](./readme_pic/github_pages.png)

Wait for the automatic deployment of the page to complete. You should then see a section like the one below on your `Pages` interface. Click `Visit site` to access your own webpage. Of course, currently the site will only display the current README content. After the first scheduled update, the website's page will transform into something like the [example](https://oshinoshinobu-chan.github.io/arxiv-sniffer/).

![visit page illustration](./readme_pic/visit_page.png)

If you want to see the effect immediately, you can also manually trigger the project once. Click on the `Actions` interface, select `Auto Update Docs And Build Site` from the left side (at this point, you should have two items here; the other is for automatically deploying GitHub Pages). Then, an interface similar to the one below will appear at the top. Click the `Run workflow` dropdown menu, and then directly click the green `Run workflow` button to manually trigger the project once. Under normal circumstances, after waiting for some time (depending on the number of topics you subscribed to, generally at least 20 minutes), a green checkmark like the one in the image below appearing indicates success. You can then check your GitHub page (you might need to refresh).

![run workflow illustration](./readme_pic/run_workflow.png)

## How to Customize Your Own Project

### Customizing the Display Website Style

You can mainly customize the style of the display website by modifying the mkdocs [configuration](./mkdocs/mkdocs.yml) and the templates used for automatically generating the pages.

For information on configuring mkdocs, please search online yourself.

For information on modifying the page generation templates, please check the website's instructions page, or [here](./mkdocs/docs/readme.md).

### Customizing the Relevance Evaluation Scale

If you wish to design your own relevance evaluation scale, you can modify it by following the example [here](./prompts/relevance_dimensions.json). The number of scale items can also be changed, but ensure the sum of weights for all items is 1.

### Customizing the Prompt

If you wish to design your own prompt, you can modify it by following the example [here](./prompts/relevance_template.txt). Below explains the meaning of each placeholder:

- `{topic}`: This will be replaced with the detailed description of the topic from your configuration file.
- `{title}`: This will be replaced with the title of the paper being evaluated.
- `{abstract}`: This will be replaced with the original abstract of the paper being evaluated.
- `{dimension_num}`: This will be replaced with the number of dimensions in the scale.
- `{dimensions}`: This will be replaced with a list of all dimension names and their corresponding descriptions.
- `{json_outputs}`: This will be replaced with the template for the final output format required from the LLM.

### Configuration File

Below, comments explain the meaning of each configuration item.

```toml
// Crawler related configuration
[crawler]
// Interval between crawl operations, currently unused
interval_secs = 10
// Timeout for crawling a single webpage. If you find your project always timing out, you can increase this.
timeout_secs = 120
subject_code = "cs"
// User-agent used by the crawler in request headers
user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0"

[prompts]
// Path to the prompts folder, containing the scale and LLM prompt template
dir = "prompts"

[filter]
// Relevance score threshold for filtering. Papers scoring above this are considered relevant to the topic.
relevance_threshold = 85
// Concurrency level for sending requests to the LLM. Can increase program speed.
eval_concurrency = 4

[[topics]]
name = "Agentic ai"
description = "Research on system-level issues in Agentic AI systems involving the collaboration of multiple agents, such as system latency, system architecture design, etc."

[[topics]]
name = "RLHF/RLVF"
description = "Research related to system-level performance optimization methods in reinforcement learning post-training systems (RLHF/RLVF) for large language models (LLMs), including optimizations for system latency, throughput, and computational resource utilization."

[ai]

// Below are LLM client settings. Refer to DeepSeek documentation for details.
[ai.models."deepseek-chat"]
provider = "deepseek"
endpoint = "https://api.deepseek.com/chat/completions"
system_prompt = "You are a helpful assistant"
timeout_secs = 60

[ai.models."deepseek-chat".request]
model = "deepseek-chat"
thinking_type = "disabled"
frequency_penalty = 0.0
max_tokens = 4096
presence_penalty = 0.0
response_format_type = "text"
stream = false
temperature = 1.0
top_p = 1.0
tool_choice = "none"
logprobs = false

[ai.models."deepseek-reasoner"]
provider = "deepseek"
endpoint = "https://api.deepseek.com/chat/completions"
system_prompt = "You are a helpful assistant"
timeout_secs = 300

[ai.models."deepseek-reasoner".request]
model = "deepseek-reasoner"
thinking_type = "enabled"
frequency_penalty = 0.0
max_tokens = 32768
presence_penalty = 0.0
response_format_type = "text"
stream = false
temperature = 1.0
top_p = 1.0
tool_choice = "none"
logprobs = false
```
## What's Next
Next, I plan to provide functionality for summarizing the screened papers and extracting key information. For example, you will be able to use a template to let the LLM extract the core innovation, experimental results, etc. <del>After all, I'm so lazy I don't even want to read the papers myself (lol).</del>

But because I'm lazy and haven't quite figured out the best way to send the paper text to the LLM, this feature might not come so soon.

## How to Contribute
If you like this project, you are welcome to support it in the following ways:

Give it a Star (top left of the project page): Stars increase the project's visibility and help more people see it.

Submit an Issue: If you find a bug or have suggestions for new features, feel free to submit an issue. However, when encountering a bug, please provide as much information as possible, otherwise I might ignore it.

Contribute Code: For the potential new features mentioned above, contributions are welcome. (After all, I am really lazy and work on this quite casually.)