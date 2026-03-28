/** RUST functions that get injected hackily into the bottom gen_code.c **/

// Forward declarations
void rustCargoFix(void);
void rustCargoFormat(void);

static void printRustLookbackFunctionSignature(FILE* out,
                                               const char* prefix, /* Can be NULL */
                                               const TA_FuncInfo* funcInfo)
{
    char funcNameBuffer[1024]; /* Not safe, but 1024 is realistic, */
    const TA_OptInputParameterInfo* optInputParamInfo;
    int i, indent;
    toLowerSnakeCase(funcInfo->name, funcNameBuffer);

    // print lookback function header
    sprintf(gTempBuf, "%spub fn %s_lookback(\n",
            prefix? prefix:"",
            funcNameBuffer);

    print(out, gTempBuf);

    // Calculate indent for parameter alignment
    indent = (unsigned int)strlen(gTempBuf);

    // Print optional input parameters - only these are needed for lookback
    for (i = 0; i < funcInfo->nbOptInput; i++)
    {
        TA_GetOptInputParameterInfo(funcInfo->handle, i, &optInputParamInfo);

        if (i > 0)
        {
            fprintf(out, ",\n");
            printIndent(out, indent);
        }

        switch (optInputParamInfo->type)
        {
        case TA_OptInput_RealRange:
            fprintf(out, "%s: f64", optInputParamInfo->paramName);
            break;
        case TA_OptInput_IntegerRange:
            fprintf(out, "%s: i32", optInputParamInfo->paramName);
            break;
        case TA_OptInput_IntegerList:
            fprintf(out, "%s: i32", optInputParamInfo->paramName);
            break;
        case TA_OptInput_RealList:
            fprintf(out, "%s: f64", optInputParamInfo->paramName);
            break;
        default:
            printf("Unknown optional input type for %s\n", funcInfo->name);
            break;
        }
    }

    // Close function and add return type
    fprintf(out, ") -> i32\n");
}

static void printRustDoublePrecisionFunctionSignature(FILE* out,
                                                      const char* prefix, /* Can be NULL */
                                                      const TA_FuncInfo* funcInfo)
{
    char funcNameBuffer[1024]; /* Not safe, but 1024 is realistic, */
    const TA_OptInputParameterInfo* optInputParamInfo;
    const TA_OutputParameterInfo* outputParamInfo;
    const TA_InputParameterInfo* inputParamInfo;
    toLowerSnakeCase(funcInfo->name, funcNameBuffer);
    int indent, i;

    // Print function header with idiomatic Rust signature
    sprintf(gTempBuf, "%spub fn %s(",
            prefix? prefix:"",
            funcNameBuffer);

    print(out, gTempBuf);

    // Calculate indent for parameter alignment
    indent = (unsigned int)strlen(gTempBuf);

    // Start with startIdx and endIdx parameters
    fprintf(out, "startIdx: usize,\n");
    printIndent(out, indent);
    fprintf(out, "endIdx: usize,\n");

    // Print input parameters with proper Rust types
    for (i = 0; i < funcInfo->nbInput; i++)
    {
        TA_GetInputParameterInfo(funcInfo->handle, i, &inputParamInfo);

        printIndent(out, indent);

        switch (inputParamInfo->type)
        {
        case TA_Input_Real:
            fprintf(out, "%s: &[f64],\n", inputParamInfo->paramName);
            break;
        case TA_Input_Integer:
            fprintf(out, "%s: &[i32],\n", inputParamInfo->paramName);
            break;
        case TA_Input_Price:
            // Handle complex price inputs - can have multiple components
            switch (inputParamInfo->flags)
            {
            case TA_IN_PRICE_OPEN:
                fprintf(out, "%s: &[f64],\n", "inOpen");
                break;
            case TA_IN_PRICE_HIGH:
                fprintf(out, "%s: &[f64],\n", "inHigh");
                break;
            case TA_IN_PRICE_LOW:
                fprintf(out, "%s: &[f64],\n", "inLow");
                break;
            case TA_IN_PRICE_CLOSE:
                fprintf(out, "%s: &[f64],\n", "inClose");
                break;
            case TA_IN_PRICE_VOLUME:
                fprintf(out, "%s: &[f64],\n", "inVolume");
                break;
            default:
                printf("Unsupported price input for %s\n", funcInfo->name);
                break;
            }
            break;
        default:
            printf("Unknown input type for %s\n", funcInfo->name);
            break;
        }
    }

    // Print optional input parameters
    for (i = 0; i < funcInfo->nbOptInput; i++)
    {
        TA_GetOptInputParameterInfo(funcInfo->handle, i, &optInputParamInfo);

        printIndent(out, indent);

        switch (optInputParamInfo->type)
        {
        case TA_OptInput_RealRange:
            fprintf(out, "%s: f64,\n", optInputParamInfo->paramName);
            break;
        case TA_OptInput_IntegerRange:
            fprintf(out, "%s: i32,\n", optInputParamInfo->paramName);
            break;
        case TA_OptInput_IntegerList:
            fprintf(out, "%s: i32,\n", optInputParamInfo->paramName);
            break;
        case TA_OptInput_RealList:
            fprintf(out, "%s: f64,\n", optInputParamInfo->paramName);
            break;
        default:
            printf("Unknown optional input type for %s\n", funcInfo->name);
            break;
        }
    }

    // Print output parameters with proper Rust mutable references
    printIndent(out, indent);
    fprintf(out, "outBegIdx: &mut usize,\n");
    printIndent(out, indent);
    fprintf(out, "outNBElement: &mut usize,\n");

    for (i = 0; i < funcInfo->nbOutput; i++)
    {
        TA_GetOutputParameterInfo(funcInfo->handle, i, &outputParamInfo);
        printIndent(out, indent);

        switch (outputParamInfo->type)
        {
        case TA_Output_Real:
            fprintf(out, "%s: &mut [f64]", outputParamInfo->paramName);
            break;
        case TA_Output_Integer:
            fprintf(out, "%s: &mut [i32]", outputParamInfo->paramName);
            break;
        default:
            printf("Unknown output type for %s\n", funcInfo->name);
            break;
        }

        // Add comma except for the last parameter
        if (i < funcInfo->nbOutput - 1)
        {
            fprintf(out, ",\n");
        }
    }

    // Close parameters and add return type
    fprintf(out, ") -> RetCode\n");
}

static void printRustSinglePrecisionFunctionSignature(FILE* out,
                                                      const char* prefix, /* Can be NULL */
                                                      const TA_FuncInfo* funcInfo)
{
    char funcNameBuffer[1024]; /* Not safe, but 1024 is realistic, */
    const TA_OptInputParameterInfo* optInputParamInfo;
    const TA_OutputParameterInfo* outputParamInfo;
    const TA_InputParameterInfo* inputParamInfo;
    toLowerSnakeCase(funcInfo->name, funcNameBuffer);
    int indent, i;

    // Print function header with idiomatic Rust signature for single precision
    sprintf(gTempBuf, "%spub fn %s_s(",
            prefix? prefix:"",
            funcNameBuffer);

    print(out, gTempBuf);

    // Calculate indent for parameter alignment
    indent = (unsigned int)strlen(gTempBuf);

    // Start with startIdx and endIdx parameters
    fprintf(out, "startIdx: usize,\n");
    printIndent(out, indent);
    fprintf(out, "endIdx: usize,\n");

    // Print input parameters with proper Rust types (f32 for single precision)
    for (i = 0; i < funcInfo->nbInput; i++)
    {
        TA_GetInputParameterInfo(funcInfo->handle, i, &inputParamInfo);

        printIndent(out, indent);

        switch (inputParamInfo->type)
        {
        case TA_Input_Real:
            fprintf(out, "%s: &[f32],\n", inputParamInfo->paramName);
            break;
        case TA_Input_Integer:
            fprintf(out, "%s: &[i32],\n", inputParamInfo->paramName);
            break;
        case TA_Input_Price:
            // Handle complex price inputs - can have multiple components
            switch (inputParamInfo->flags)
            {
            case TA_IN_PRICE_OPEN:
                fprintf(out, "%s: &[f32],\n", "inOpen");
                break;
            case TA_IN_PRICE_HIGH:
                fprintf(out, "%s: &[f32],\n", "inHigh");
                break;
            case TA_IN_PRICE_LOW:
                fprintf(out, "%s: &[f32],\n", "inLow");
                break;
            case TA_IN_PRICE_CLOSE:
                fprintf(out, "%s: &[f32],\n", "inClose");
                break;
            case TA_IN_PRICE_VOLUME:
                fprintf(out, "%s: &[f32],\n", "inVolume");
                break;
            default:
                printf("Unsupported price input for %s\n", funcInfo->name);
                break;
            }
            break;
        default:
            printf("Unknown input type for %s\n", funcInfo->name);
            break;
        }
    }

    // Print optional input parameters
    for (i = 0; i < funcInfo->nbOptInput; i++)
    {
        TA_GetOptInputParameterInfo(funcInfo->handle, i, &optInputParamInfo);

        printIndent(out, indent);

        switch (optInputParamInfo->type)
        {
        case TA_OptInput_RealRange:
            fprintf(out, "%s: f32,\n", optInputParamInfo->paramName);
            break;
        case TA_OptInput_IntegerRange:
            fprintf(out, "%s: i32,\n", optInputParamInfo->paramName);
            break;
        case TA_OptInput_IntegerList:
            fprintf(out, "%s: i32,\n", optInputParamInfo->paramName);
            break;
        case TA_OptInput_RealList:
            fprintf(out, "%s: f32,\n", optInputParamInfo->paramName);
            break;
        default:
            printf("Unknown optional input type for %s\n", funcInfo->name);
            break;
        }
    }

    // Print output parameters with proper Rust mutable references
    // Note: outputs still use f64 even for single precision functions
    printIndent(out, indent);
    fprintf(out, "outBegIdx: &mut usize,\n");
    printIndent(out, indent);
    fprintf(out, "outNBElement: &mut usize,\n");

    for (i = 0; i < funcInfo->nbOutput; i++)
    {
        TA_GetOutputParameterInfo(funcInfo->handle, i, &outputParamInfo);
        printIndent(out, indent);

        switch (outputParamInfo->type)
        {
        case TA_Output_Real:
            fprintf(out, "%s: &mut [f64]", outputParamInfo->paramName);
            break;
        case TA_Output_Integer:
            fprintf(out, "%s: &mut [i32]", outputParamInfo->paramName);
            break;
        default:
            printf("Unknown output type for %s\n", funcInfo->name);
            break;
        }

        // Add comma except for the last parameter
        if (i < funcInfo->nbOutput - 1)
        {
            fprintf(out, ",\n");
        }
    }

    // Close parameters and add return type
    fprintf(out, ") -> RetCode\n");
}

void writeRustModLines(const TA_FuncInfo* funcInfo, void* opaque)
{
    struct WriteRustModLinesParams* params = (struct WriteRustModLinesParams*)opaque;
    FileHandle* out = params->out;
    char buffer[500];

#if defined(RUST_SINGLE_FUNC)
    if (strcmp(funcInfo->name,RUST_SINGLE_FUNC) != 0)
        return;
#endif

    // Convert filename to lowercase into buffer.
    int i = 0;
    while (funcInfo->name[i])
    {
        buffer[i] = tolower(funcInfo->name[i]);
        i++;
    }
    buffer[i] = '\0';

    if (params->writePubUse == 1)
        fprintf(out->file, "pub use self::%s::*;\n", buffer);
    else
        fprintf(out->file, "pub mod %s;\n", buffer);
}

void writeRustMod(void)
{
    // Update the rust/src/ta_func/mod.rs file.
    struct WriteRustModLinesParams params;
    char buffer[500];
    FileHandle* out;

    if (!gmcpp_installed)
        return;

    // Add rs file to ta_func/mod.rs
    // A common header/footer is provided by the template file.
#define FILE_RUST_MOD ".." PATH_SEPARATOR "rust" PATH_SEPARATOR "src" PATH_SEPARATOR "ta_func" PATH_SEPARATOR "mod.rs"
#define FILE_RUST_MOD_TEMPLATE ".." PATH_SEPARATOR "src" PATH_SEPARATOR "ta_abstract" PATH_SEPARATOR "templates" PATH_SEPARATOR "ta_func_mod.rs.template"

    out = fileOpen(FILE_RUST_MOD,
                   FILE_RUST_MOD_TEMPLATE,
                   FILE_WRITE | WRITE_ON_CHANGE_ONLY);


    params.out = out;
    params.writePubUse = 1;
    TA_ForEachFunc(writeRustModLines, &params);
    params.writePubUse = 0;
    TA_ForEachFunc(writeRustModLines, &params);

    fileClose(out);
    
    // Run cargo commands to clean up generated Rust code
    rustCargoFix();
    rustCargoFormat();
}

void genRustCodePhase2(const TA_FuncInfo* funcInfo)
{
    // Each TA function get its own .rs file generated.
    // A common header/footer is provided by the template file.
    FILE* logicTmp;
    char buffer[500];
    int idx, again;
    static int firstTime = 1;
    int ret;

#if defined(RUST_SINGLE_FUNC)
    if (strcmp(funcInfo->name,RUST_SINGLE_FUNC) != 0)
        return;
#endif

    if (!gmcpp_installed)
        return;


    // Convert filename to lowercase into buffer.
    int i = 0;
    while (funcInfo->name[i])
    {
        buffer[i] = tolower(funcInfo->name[i]);
        i++;
    }
    buffer[i] = '\0';
    strcat(buffer, ".rs");
#define FILE_RUST_FUNC_TEMPLATE ".." PATH_SEPARATOR "src" PATH_SEPARATOR "ta_abstract" PATH_SEPARATOR "templates" PATH_SEPARATOR "ta_x.rs.template"

    FileHandle* out = fileOpen(ta_fs_path(5, "..", "rust", "src", "ta_func", buffer),
                               FILE_RUST_FUNC_TEMPLATE,
                               FILE_WRITE | WRITE_ON_CHANGE_ONLY);


    /* Clean-up just in case. */
    fileDelete(ta_fs_path(3, "..", "temp", "rust_logic.tmp"));

#ifdef _MSC_VER
     sprintf( buffer, "%s -c -+ -z -P -I.." PATH_SEPARATOR "src" PATH_SEPARATOR "ta_common -I.." PATH_SEPARATOR "src" PATH_SEPARATOR "ta_abstract -I.." PATH_SEPARATOR "include -D _RUST .." PATH_SEPARATOR "src" PATH_SEPARATOR "ta_func" PATH_SEPARATOR "TA_%s.c >>.." PATH_SEPARATOR "temp" PATH_SEPARATOR "rust_logic.tmp ", gmcpp_exec, funcInfo->name);
     ret = system( buffer );
#else
    sprintf(buffer,
            "%s -@compat -+ -z -P -I.." PATH_SEPARATOR "src" PATH_SEPARATOR "ta_common -I.." PATH_SEPARATOR "src"
            PATH_SEPARATOR "ta_abstract -I.." PATH_SEPARATOR "include -D _RUST .." PATH_SEPARATOR "src" PATH_SEPARATOR
            "ta_func" PATH_SEPARATOR "ta_%s.c | sed '/^#include/d' >> .." PATH_SEPARATOR "temp" PATH_SEPARATOR
            "rust_logic.tmp ", gmcpp_exec, funcInfo->name);
    ret = system(buffer);
#endif

    /* Write the output of the C pre-processor to the rust file. */
    init_gToOpen(ta_fs_path(3, "..", "temp", "rust_logic.tmp"), NULL);
    logicTmp = fopen(gToOpen, "r");
    if (!logicTmp)
    {
        printf("Cannot open temp/rust_logic.tmp\n");
        return;
    }
    while (fgets(gTempBuf,BUFFER_SIZE, logicTmp))
    {
        /* Remove empty lines and lines with only a ';' */
        idx = 0;
        again = 1;
        while (again && gTempBuf[idx] != '\0')
        {
            if (!isspace(gTempBuf[idx]) && !(gTempBuf[idx] == ';'))
                again = 0;
            idx++;
        }
        if ((again == 0) && (idx > 0))
            fputs(gTempBuf, out->file);
    }

    /* Clean-up */
    fclose(logicTmp);
    print(out->file, "\n");
    fileDelete(ta_fs_path(3, "..", "temp", "rust_logic.tmp"));

    // Upon closing, will touch the target file only if there was a change...
    fileClose(out);
}

static int tryCargoCommand(const char* command)
{
    char fullCommand[1024];
    int result;
    
    // Try 1: Use PATH (standard installation)
    snprintf(fullCommand, sizeof(fullCommand), "cd " ".." PATH_SEPARATOR "rust" " && cargo %s 2>/dev/null", command);
    result = system(fullCommand);
    if (result == 0) return 0;
    
    // Try 2: Common user installation location
    snprintf(fullCommand, sizeof(fullCommand), "cd " ".." PATH_SEPARATOR "rust" " && ~/.cargo/bin/cargo %s 2>/dev/null", command);
    result = system(fullCommand);
    if (result == 0) return 0;
    
    // Try 3: System-wide installation (Linux/macOS)
    snprintf(fullCommand, sizeof(fullCommand), "cd " ".." PATH_SEPARATOR "rust" " && /usr/local/bin/cargo %s 2>/dev/null", command);
    result = system(fullCommand);
    if (result == 0) return 0;
    
    // Try 4: Check if CARGO_HOME is set
    const char* cargoHome = getenv("CARGO_HOME");
    if (cargoHome) {
        snprintf(fullCommand, sizeof(fullCommand), "cd " ".." PATH_SEPARATOR "rust" " && %s/bin/cargo %s 2>/dev/null", cargoHome, command);
        result = system(fullCommand);
        if (result == 0) return 0;
    }
    
    return -1; // All attempts failed
}

void rustCargoFix(void)
{
    printf("Running cargo fix on generated Rust code...\n");
    
    if (tryCargoCommand("fix --lib -p ta-lib --allow-dirty") != 0) {
        printf("Warning: cargo fix failed (cargo not found or command failed)\n");
        printf("  To enable automatic cargo fix, ensure 'cargo' is in your PATH\n");
    }
}

void rustCargoFormat(void)
{
    printf("Running cargo fmt on generated Rust code...\n");
    
    if (tryCargoCommand("fmt") != 0) {
        printf("Warning: cargo fmt failed (cargo not found or command failed)\n");
        printf("  To enable automatic cargo fmt, ensure 'cargo' is in your PATH\n");
    }
}
