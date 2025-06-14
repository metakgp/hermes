<div id="top"></div>

<!-- PROJECT SHIELDS -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links-->
<div align="center">

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![AGPL-3.0 License][license-shield]][license-url]
[![Wiki][wiki-shield]][wiki-url]

</div>

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/metakgp/hermes">
     <img width="140" alt="image" src="https://raw.githubusercontent.com/metakgp/design/main/logos/black-large.jpg">
  </a>

  <h3 align="center">HERMES</h3>

  <p align="center">
    <i>High speed Encrypted and Real-time Messaging and Efficient Sharing</i>
    <br />
    <i>A modern P2P replacement for DC++ on KGP networks</i>
    <br />
    <a href="https://github.com/metakgp/hermes/issues">Request Feature / Report Bug</a>
  </p>
</div>


<!-- TABLE OF CONTENTS -->
<details>
<summary>Table of Contents</summary>

- [About The Project](#about-the-project)
  - [Architecture](#architecture)
- [Development Setup](#development-setup)
  - [Host](#host)
  - [Peer](#peer)
- [Usage](#usage)
- [Contact](#contact)
  - [Maintainer(s)](#maintainers)
- [Additional documentation](#additional-documentation)

</details>


<!-- ABOUT THE PROJECT -->
## About The Project

**HERMES** is a work-in-progress project aimed at modernizing and eventually replacing the legacy DC++ file sharing system used on KGP network. It is being developed using Tauri and Svelte, with a decentralized peer-to-peer architecture powered by [iroh](https://iroh.computer/docs/)

> [!Note]
> Currently in active development. Get involved at our [Slack](https://slack.metakgp.org/).

### Architecture

The project consists of two main components:

1. **Host** (`host/` folder): 
   - Central server component hosted on campus Raspberry Pi
   - Handles user registration, nickname management, and moderation

2. **Peer** (`peer/` folder):
   - Cross-platform desktop client application
   - Built with Tauri (Rust backend) + Svelte (TypeScript frontend)
   - Handles direct P2P file transfers and messaging
   - Uses iroh for networking and NAT traversal

<p align="right">(<a href="#top">back to top</a>)</p>

## Development Setup
To set up a local development environment for HERMES, follow the steps below.

1. Clone the repository
   ```sh
   git clone https://github.com/metakgp/hermes.git
   ```

2. Navigate to the project directory
   ```sh
   cd hermes
   ```

3. Follow the development setup instructions below for your component of interest.

### Host
> **TODO**: Host development setup instructions will be added as the server component is developed.

### Peer

#### Prerequisites
- **pnpm**
- **Rust**
- **Tauri dependencies** – Refer to the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/)


#### Development
1. Navigate to the peer directory
   ```sh
   cd peer
   ```

2. Install dependencies
   ```sh
   pnpm install
   ```

3. Start the development server
   ```sh
   pnpm tauri dev
   ```

This will start the Tauri development environment with hot-reload enabled for both the Rust backend and Svelte frontend.

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- USAGE EXAMPLES -->
## Usage

HERMES is currently in active development. Once released, it will provide:

- **File Sharing**: Direct P2P file transfers between campus users
- **Real-time Chat**: Instant messaging with connected peers

More usage examples and screenshots will be added as the project progresses.

<p align="right">(<a href="#top">back to top</a>)</p>

## Contact

<p>
📫 Metakgp -
<a href="https://slack.metakgp.org">
  <img align="center" alt="Metakgp's slack invite" width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/slack.svg" />
</a>
<a href="mailto:metakgp@gmail.com">
  <img align="center" alt="Metakgp's email " width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/gmail.svg" />
</a>
<a href="https://www.facebook.com/metakgp">
  <img align="center" alt="metakgp's Facebook" width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/facebook.svg" />
</a>
<a href="https://www.linkedin.com/company/metakgp-org/">
  <img align="center" alt="metakgp's LinkedIn" width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/linkedin.svg" />
</a>
<a href="https://twitter.com/metakgp">
  <img align="center" alt="metakgp's Twitter " width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/twitter.svg" />
</a>
<a href="https://www.instagram.com/metakgp_/">
  <img align="center" alt="metakgp's Instagram" width="22px" src="https://raw.githubusercontent.com/edent/SuperTinyIcons/master/images/svg/instagram.svg" />
</a>
</p>

### Maintainer(s)

The currently active maintainer(s) of this project.

- [Yogansh Sharma](https://github.com/YoganshSharma)
- [Jeffrey Samuel](https://github.com/Signor-Koala)

<p align="right">(<a href="#top">back to top</a>)</p>

## Additional documentation

  - [License](/LICENSE)
  - [Code of Conduct](/.github/CODE_OF_CONDUCT.md)
  - [Security Policy](/.github/SECURITY.md)
  - [Contribution Guidelines](/.github/CONTRIBUTING.md)

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->

[contributors-shield]: https://img.shields.io/github/contributors/metakgp/hermes.svg?style=for-the-badge
[contributors-url]: https://github.com/metakgp/hermes/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/metakgp/hermes.svg?style=for-the-badge
[forks-url]: https://github.com/metakgp/hermes/network/members
[stars-shield]: https://img.shields.io/github/stars/metakgp/hermes.svg?style=for-the-badge
[stars-url]: https://github.com/metakgp/hermes/stargazers
[issues-shield]: https://img.shields.io/github/issues/metakgp/hermes.svg?style=for-the-badge
[issues-url]: https://github.com/metakgp/hermes/issues
[license-shield]: https://img.shields.io/github/license/metakgp/hermes.svg?style=for-the-badge
[license-url]: https://github.com/metakgp/hermes/blob/master/LICENSE
[wiki-shield]: https://custom-icon-badges.demolab.com/badge/metakgp_wiki-grey?logo=metakgp_logo&style=for-the-badge
[wiki-url]: https://wiki.metakgp.org
[slack-url]: https://slack.metakgp.org
