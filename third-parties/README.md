# Third-Party Dependencies

This directory contains forked and optimized third-party dependencies.

## Adding a New Dependency

1. **Clone the forked repository:**
   ```bash
   git clone <repository-url>
   ```

2. **Add it to the workspace:**
   - Ensure the dependency has a valid `package.json` file.
   - The package will be automatically included in the workspace thanks to the configuration in the root `package.json`.

3. **Install dependencies:**
   - Run `bun install` from the root of the project to link the local package.

4. **Use the local package:**
   - In the `package.json` of the package where you want to use the forked dependency, add or update the dependency entry to point to the local version, for example:
     ```json
     "dependencies": {
       "my-forked-dependency": "workspace:*"
     }
     ```
