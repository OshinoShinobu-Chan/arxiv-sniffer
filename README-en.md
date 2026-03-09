# ArXiv Sniffer

> Note: This README is translated from chinese version, totally by DeepSeek.

## Introduction

This project is a Large Language Model (LLM)-based program designed to filter and summarize papers from the arXiv catchup (arXiv's daily subscription) page.

## Current Features

> Note: This project is currently in its early development stage and does not yet provide a user-facing external interface.

### Crawling ArXiv Catchup

ArXiv catchup is a daily subscription page provided by the arXiv website. Each research field has a daily page listing all new submissions and recent revisions for that day.

This project provides functionality to crawl all **new submission** paper entries from the arXiv catchup for a given date. It reads the arXiv ID, title, authors, and abstract of these papers.

### Filtering Papers

This project offers paper filtering capabilities based on LLMs. It filters papers relevant to your specified topics by analyzing their titles and abstracts.

The specific filtering method involves designing a multi-dimensional relevance scale ([example](./prompts/relevance_dimensions.json)). Information for each paper is inserted into a prompt template ([example](./prompts/relevance_template.txt)) and sent to an LLM via its API. The LLM scores the paper according to the scale, and a total score is calculated. If the score exceeds a certain threshold, the paper is considered relevant to your topic of interest.

Additionally, the LLM is prompted to provide reasoning for its scores on each dimension for your reference.

## Planned Features

> The following features are planned for development, listed in my estimated order of implementation. I will prioritize features at the top of this list.

### Downloading Full Papers from arXiv

Using the obtained arXiv IDs, this feature will enable downloading the full text of papers from arXiv. The project will support downloading both PDF and TeX source formats. This functionality primarily supports the features described below.

### Evaluating and Summarizing Papers

This project will support evaluating and summarizing paper content using LLMs.

#### Evaluating Papers

Paper evaluation will be similar to the filtering process described above. A scale for assessing paper quality will be designed, and the LLM will be asked to score papers accordingly. An additional feature could prompt the LLM to provide a critique and suggestions for improvement. This aims to help users further identify papers worthy of in-depth reading.

#### Summarizing Paper Content

For papers that are relevant and receive high quality scores, the LLM can be asked to summarize the key points from the full text. The summarization method involves preparing a prompt template (e.g., a Markdown document with placeholders for key information) for the LLM to fill in. This feature aims to further enhance reading efficiency, allowing users to focus on sections of genuine interest based on the summary, without always reading the entire paper.

### Methods for Sending Papers to the AI

<del>As a devoted DeepSeek user</del>. Since the DeepSeek API does not currently support file uploads, we plan to provide at least four methods for sending the full paper text to the LLM.

#### Sending TeX Source to the LLM

- **Advantage**: Expected to handle mathematical formulas well.
- **Disadvantage**: Contains significant extraneous information, which can waste tokens and potentially interfere with the LLM's judgment.

#### Using Local OCR on PDF Files

- **Advantage**: Avoids the disadvantages of sending TeX source.
- **Disadvantages**:
    1.  Rust currently lacks robust, easy-to-use OCR libraries.
    2.  Processing long or numerous papers could put significant strain on your local machine.
    3.  Handling of mathematical formulas is uncertain.

#### Using Models that Support File Uploads

- **Advantage**: Native support might yield better results.
- **Disadvantage**: API costs are likely higher compared to DeepSeek.

#### Using a Web Crawler to Upload Files to DeepSeek via its Web Interface

- **Advantages**:
    1.  Proven effectiveness.
    2.  Free of charge.
- **Disadvantage**: This is a non-standard approach and may be unstable.

### Implementing a User Interface

The exact mode is not yet finalized, but several ideas are outlined below. The project's execution and deployment methods, as well as the ways to display daily filtering results, each have several options that can be combined arbitrarily.

> Below are several ways to run the project itself.

#### Scheduled Task

Deploy this project as a scheduled task (e.g., cron job) on your computer, running daily at a specific time.

- **Advantages**:
    1.  Simple to implement.
    2.  Daily crawling and filtering run silently in the background; no waiting required.
- **Disadvantages**:
    1.  Requires your computer to be on at the scheduled time each day.

#### Running as a Service

Register this project as a service on your computer. It will start automatically with the system and run silently in the background. If running, it will perform crawling and filtering at an appropriate time each day. The system load is negligible when idle.

- **Advantages**:
    1.  Daily crawling and filtering run silently in the background; no waiting required.
- **Disadvantages**:
    1.  Registering as a service involves some technical steps.
    2.  Requires higher stability from the project.

#### On-Demand Execution via Web UI

Package this project as a program with a web UI. You start it manually when you need to perform crawling and filtering.

- **Advantages**:
    1.  Simple interaction model.
- **Disadvantages**:
    1.  Requires waiting for crawling and filtering processes after startup.

> Below are several ways to display the results.

#### Display via File Output

Write the daily crawling and filtering results to files for the user to view.

- **Advantages**:
    1.  Simple to implement.
- **Disadvantages**:
    1.  Requires more steps for the user to access results.
    2.  Lacks a clear UI.

#### Automatic Deployment via GitHub Pages

Deploy the daily results automatically using GitHub Pages.

- **Advantages**:
    1.  Easy to use.
    2.  Allows for a better UI.
    3.  Easier to share results with others.
    4.  Can be deployed on a remote server.
- **Disadvantages**:
    1.  Requires a GitHub account.

#### Using a Web UI

Integrate results and interactions into a web UI.

- **Advantages**:
    1.  Allows for more complex interactions.
    2.  Allows for a better UI.
    3.  Can be deployed on a remote server.
- **Disadvantages**:
    1.  Complex to implement.
    2.  May require more steps for users to access.

#### Using a Pure Command-Line Interface

Display results using a pure command-line interface (CLI).

- **Advantages**:
    1.  Simple to implement.
- **Disadvantages**:
    1.  Results display may not be clear.
    2.  Presents a barrier to entry for some users.

### Supporting Other Model APIs

As the title suggests.

> The following are ideas that are currently speculative and may not be implemented.

### Generating Research Ideas

Based on the summaries of read papers, prompt the LLM to generate research ideas using a template. The template could represent a general scientific methodology for your specific field or topic of interest. The LLM could then generate ideas following this methodology.

### Supporting More General Crawlers

Support crawling from other similar preprint servers. I haven't figured out the specifics for this yet :(.