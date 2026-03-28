#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ta_common.h"
#include "ta_func.h"

static void print_result(TA_RetCode code, int outBegIdx, int outNbElement, const double *outReal) {
    printf("%d %d %d", (int)code, outBegIdx, outNbElement);
    for (int i = 0; i < outNbElement; i++) {
        printf(" %.17g", outReal[i]);
    }
    printf("\n");
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "expected one case argument\n");
        return 2;
    }

    TA_RetCode code = TA_Initialize();
    if (code != TA_SUCCESS) {
        fprintf(stderr, "TA_Initialize failed: %d\n", (int)code);
        return 3;
    }

    int outBegIdx = 0;
    int outNbElement = 0;
    double outReal[256] = {0.0};

    if (strcmp(argv[1], "add_basic") == 0) {
        const double inReal0[] = {1.0, 2.0, 3.0, 4.0, 5.0};
        const double inReal1[] = {5.0, 4.0, 3.0, 2.0, 1.0};
        code = TA_ADD(0, 4, inReal0, inReal1, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "sub_basic") == 0) {
        const double inReal0[] = {10.0, 9.0, 8.0, 7.0, 6.0};
        const double inReal1[] = {1.0, 2.0, 3.0, 4.0, 5.0};
        code = TA_SUB(0, 4, inReal0, inReal1, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "mult_basic") == 0) {
        const double inReal0[] = {1.0, 2.0, 3.0, 4.0, 5.0};
        const double inReal1[] = {2.0, 3.0, 4.0, 5.0, 6.0};
        code = TA_MULT(0, 4, inReal0, inReal1, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "div_basic") == 0) {
        const double inReal0[] = {10.0, 9.0, 8.0, 7.0, 6.0};
        const double inReal1[] = {2.0, 3.0, 4.0, 5.0, 6.0};
        code = TA_DIV(0, 4, inReal0, inReal1, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "sma_period_3") == 0) {
        const double inReal[] = {1.0, 2.0, 3.0, 4.0, 5.0, 6.0};
        code = TA_SMA(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ema_default") == 0) {
        const double inReal[] = {1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0};
        code = TA_EMA(0, 6, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ema_metastock") == 0) {
        const double inReal[] = {1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0};
        TA_SetCompatibility(TA_COMPATIBILITY_METASTOCK);
        code = TA_EMA(0, 6, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ema_unstable_2") == 0) {
        const double inReal[] = {1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0};
        TA_SetUnstablePeriod(TA_FUNC_UNST_EMA, 2);
        code = TA_EMA(0, 7, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "acos_basic") == 0) {
        const double inReal[] = {-1.0, -0.5, 0.0, 0.5, 1.0};
        code = TA_ACOS(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "asin_basic") == 0) {
        const double inReal[] = {-1.0, -0.5, 0.0, 0.5, 1.0};
        code = TA_ASIN(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "atan_basic") == 0) {
        const double inReal[] = {-2.0, -1.0, 0.0, 1.0, 2.0};
        code = TA_ATAN(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ceil_basic") == 0) {
        const double inReal[] = {-1.7, -0.2, 0.0, 1.2, 2.8};
        code = TA_CEIL(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "cos_basic") == 0) {
        const double inReal[] = {-1.0, -0.5, 0.0, 0.5, 1.0};
        code = TA_COS(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "cosh_basic") == 0) {
        const double inReal[] = {-1.0, -0.5, 0.0, 0.5, 1.0};
        code = TA_COSH(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "exp_basic") == 0) {
        const double inReal[] = {-1.0, 0.0, 1.0, 2.0, 3.0};
        code = TA_EXP(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "floor_basic") == 0) {
        const double inReal[] = {-1.7, -0.2, 0.0, 1.2, 2.8};
        code = TA_FLOOR(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ln_basic") == 0) {
        const double inReal[] = {1.0, 2.0, 4.0, 8.0, 16.0};
        code = TA_LN(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "sqrt_basic") == 0) {
        const double inReal[] = {1.0, 4.0, 9.0, 16.0, 25.0};
        code = TA_SQRT(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "log10_basic") == 0) {
        const double inReal[] = {1.0, 10.0, 100.0, 1000.0, 10000.0};
        code = TA_LOG10(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "sin_basic") == 0) {
        const double inReal[] = {-1.0, -0.5, 0.0, 0.5, 1.0};
        code = TA_SIN(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "sinh_basic") == 0) {
        const double inReal[] = {-1.0, -0.5, 0.0, 0.5, 1.0};
        code = TA_SINH(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "tan_basic") == 0) {
        const double inReal[] = {-1.0, -0.5, 0.0, 0.5, 1.0};
        code = TA_TAN(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "tanh_basic") == 0) {
        const double inReal[] = {-1.0, -0.5, 0.0, 0.5, 1.0};
        code = TA_TANH(0, 4, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "sum_period_3") == 0) {
        const double inReal[] = {1.0, 2.0, 3.0, 4.0, 5.0, 6.0};
        code = TA_SUM(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "max_period_3") == 0) {
        const double inReal[] = {1.0, 7.0, 3.0, 4.0, 6.0, 2.0};
        code = TA_MAX(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "min_period_3") == 0) {
        const double inReal[] = {5.0, 2.0, 3.0, 1.0, 6.0, 4.0};
        code = TA_MIN(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "maxindex_period_3") == 0) {
        const double inReal[] = {1.0, 7.0, 3.0, 4.0, 6.0, 2.0};
        int outInteger[64] = {0};
        code = TA_MAXINDEX(0, 5, inReal, 3, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) {
            printf(" %d", outInteger[i]);
        }
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "minindex_period_3") == 0) {
        const double inReal[] = {5.0, 2.0, 3.0, 1.0, 6.0, 4.0};
        int outInteger[64] = {0};
        code = TA_MININDEX(0, 5, inReal, 3, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) {
            printf(" %d", outInteger[i]);
        }
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "minmax_period_3") == 0) {
        const double inReal[] = {5.0, 2.0, 7.0, 1.0, 6.0, 4.0};
        double outMin[64] = {0.0};
        double outMax[64] = {0.0};
        code = TA_MINMAX(0, 5, inReal, 3, &outBegIdx, &outNbElement, outMin, outMax);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) {
            printf(" %.17g", outMin[i]);
        }
        printf(" |");
        for (int i = 0; i < outNbElement; i++) {
            printf(" %.17g", outMax[i]);
        }
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "minmaxindex_period_3") == 0) {
        const double inReal[] = {5.0, 2.0, 7.0, 1.0, 6.0, 4.0};
        int outMinIdx[64] = {0};
        int outMaxIdx[64] = {0};
        code = TA_MINMAXINDEX(0, 5, inReal, 3, &outBegIdx, &outNbElement, outMinIdx, outMaxIdx);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) {
            printf(" %d", outMinIdx[i]);
        }
        printf(" |");
        for (int i = 0; i < outNbElement; i++) {
            printf(" %d", outMaxIdx[i]);
        }
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "avgprice_basic") == 0) {
        const double inOpen[] = {1.0, 2.0, 3.0, 4.0};
        const double inHigh[] = {2.0, 3.0, 4.0, 5.0};
        const double inLow[] = {0.5, 1.5, 2.5, 3.5};
        const double inClose[] = {1.5, 2.5, 3.5, 4.5};
        code = TA_AVGPRICE(0, 3, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "medprice_basic") == 0) {
        const double inHigh[] = {2.0, 3.0, 4.0, 5.0};
        const double inLow[] = {0.5, 1.5, 2.5, 3.5};
        code = TA_MEDPRICE(0, 3, inHigh, inLow, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "typprice_basic") == 0) {
        const double inHigh[] = {2.0, 3.0, 4.0, 5.0};
        const double inLow[] = {0.5, 1.5, 2.5, 3.5};
        const double inClose[] = {1.5, 2.5, 3.5, 4.5};
        code = TA_TYPPRICE(0, 3, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "wclprice_basic") == 0) {
        const double inHigh[] = {2.0, 3.0, 4.0, 5.0};
        const double inLow[] = {0.5, 1.5, 2.5, 3.5};
        const double inClose[] = {1.5, 2.5, 3.5, 4.5};
        code = TA_WCLPRICE(0, 3, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "midpoint_period_3") == 0) {
        const double inReal[] = {5.0, 2.0, 7.0, 1.0, 6.0, 4.0};
        code = TA_MIDPOINT(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "midprice_period_3") == 0) {
        const double inHigh[] = {5.0, 7.0, 6.0, 8.0, 4.0, 9.0};
        const double inLow[] = {1.0, 2.0, 3.0, 2.5, 1.5, 4.0};
        code = TA_MIDPRICE(0, 5, inHigh, inLow, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "trange_basic") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0};
        code = TA_TRANGE(0, 4, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "avgdev_period_3") == 0) {
        const double inReal[] = {1.0, 2.0, 3.0, 5.0, 8.0, 13.0};
        code = TA_AVGDEV(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "bop_basic") == 0) {
        const double inOpen[] = {10.0, 11.0, 10.0, 14.0, 13.0};
        const double inHigh[] = {12.0, 13.0, 11.0, 15.0, 14.0};
        const double inLow[] = {9.0, 10.0, 9.5, 13.0, 12.0};
        const double inClose[] = {11.0, 12.0, 10.5, 13.5, 13.0};
        code = TA_BOP(0, 4, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "imi_period_3") == 0) {
        const double inOpen[] = {10.0, 11.0, 10.5, 12.0, 13.0, 12.5};
        const double inClose[] = {11.0, 10.5, 11.5, 13.0, 12.0, 13.5};
        code = TA_IMI(0, 5, inOpen, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "imi_period_3_unstable_2") == 0) {
        const double inOpen[] = {10.0, 11.0, 10.5, 12.0, 13.0, 12.5, 13.5, 14.0};
        const double inClose[] = {11.0, 10.5, 11.5, 13.0, 12.0, 13.5, 14.5, 13.0};
        TA_SetUnstablePeriod(TA_FUNC_UNST_IMI, 2);
        code = TA_IMI(0, 7, inOpen, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "mom_period_3") == 0) {
        const double inReal[] = {10.0, 11.0, 12.0, 15.0, 18.0, 21.0};
        code = TA_MOM(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "roc_period_3") == 0) {
        const double inReal[] = {10.0, 11.0, 12.0, 15.0, 18.0, 21.0};
        code = TA_ROC(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "rocp_period_3") == 0) {
        const double inReal[] = {10.0, 11.0, 12.0, 15.0, 18.0, 21.0};
        code = TA_ROCP(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "rocr_period_3") == 0) {
        const double inReal[] = {10.0, 11.0, 12.0, 15.0, 18.0, 21.0};
        code = TA_ROCR(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "rocr100_period_3") == 0) {
        const double inReal[] = {10.0, 11.0, 12.0, 15.0, 18.0, 21.0};
        code = TA_ROCR100(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "atr_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0};
        code = TA_ATR(0, 5, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "atr_unstable_2") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5};
        TA_SetUnstablePeriod(TA_FUNC_UNST_ATR, 2);
        code = TA_ATR(0, 7, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "natr_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0};
        code = TA_NATR(0, 5, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "natr_unstable_2") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5};
        TA_SetUnstablePeriod(TA_FUNC_UNST_NATR, 2);
        code = TA_NATR(0, 7, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ad_basic") == 0) {
        const double inHigh[] = {12.0, 13.0, 11.0, 15.0, 14.0};
        const double inLow[] = {9.0, 10.0, 9.5, 13.0, 12.0};
        const double inClose[] = {11.0, 12.0, 10.5, 13.5, 13.0};
        const double inVolume[] = {100.0, 110.0, 120.0, 130.0, 140.0};
        code = TA_AD(0, 4, inHigh, inLow, inClose, inVolume, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "obv_basic") == 0) {
        const double inReal[] = {10.0, 11.0, 10.5, 12.0, 11.5};
        const double inVolume[] = {100.0, 110.0, 120.0, 130.0, 140.0};
        code = TA_OBV(0, 4, inReal, inVolume, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "willr_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0};
        code = TA_WILLR(0, 5, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "aroon_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0};
        double outSecond[64] = {0.0};
        code = TA_AROON(0, 6, inHigh, inLow, 3, &outBegIdx, &outNbElement, outReal, outSecond);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outSecond[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "aroonosc_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0};
        code = TA_AROONOSC(0, 6, inHigh, inLow, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "var_period_3") == 0) {
        const double inReal[] = {1.0, 2.0, 3.0, 5.0, 8.0, 13.0};
        code = TA_VAR(0, 5, inReal, 3, 1.0, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "stddev_period_3") == 0) {
        const double inReal[] = {1.0, 2.0, 3.0, 5.0, 8.0, 13.0};
        code = TA_STDDEV(0, 5, inReal, 3, 1.0, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "correl_period_3") == 0) {
        const double inReal0[] = {1.0, 2.0, 3.0, 5.0, 8.0, 13.0};
        const double inReal1[] = {2.0, 4.0, 6.0, 10.0, 16.0, 26.0};
        code = TA_CORREL(0, 5, inReal0, inReal1, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "beta_period_3") == 0) {
        const double inReal0[] = {10.0, 11.0, 12.0, 15.0, 18.0, 21.0};
        const double inReal1[] = {20.0, 21.0, 22.0, 24.0, 27.0, 30.0};
        code = TA_BETA(0, 5, inReal0, inReal1, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "cci_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0};
        code = TA_CCI(0, 5, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "cmo_default") == 0) {
        const double inReal[] = {
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
            45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
            46.03, 46.41, 46.22, 45.64, 46.21
        };
        code = TA_CMO(0, 20, inReal, 14, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "cmo_metastock") == 0) {
        const double inReal[] = {
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
            45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
            46.03, 46.41, 46.22, 45.64, 46.21
        };
        TA_SetCompatibility(TA_COMPATIBILITY_METASTOCK);
        code = TA_CMO(0, 20, inReal, 14, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "cmo_unstable_2") == 0) {
        const double inReal[] = {
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
            45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
            46.03, 46.41, 46.22, 45.64, 46.21, 46.25, 46.50
        };
        TA_SetUnstablePeriod(TA_FUNC_UNST_CMO, 2);
        code = TA_CMO(0, 22, inReal, 14, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "mfi_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0};
        const double inVolume[] = {100.0, 110.0, 120.0, 130.0, 140.0, 150.0};
        code = TA_MFI(0, 5, inHigh, inLow, inClose, inVolume, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "mfi_unstable_2") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5};
        const double inVolume[] = {100.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0, 170.0};
        TA_SetUnstablePeriod(TA_FUNC_UNST_MFI, 2);
        code = TA_MFI(0, 7, inHigh, inLow, inClose, inVolume, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "adosc_3_10") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0, 19.0, 20.0, 21.0, 22.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0, 16.0, 17.0, 18.0, 19.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5, 18.0, 19.0, 20.0, 21.0};
        const double inVolume[] = {100.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0, 170.0, 180.0, 190.0, 200.0, 210.0};
        code = TA_ADOSC(0, 11, inHigh, inLow, inClose, inVolume, 3, 10, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "plus_dm_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0};
        code = TA_PLUS_DM(0, 6, inHigh, inLow, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "minus_dm_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0};
        code = TA_MINUS_DM(0, 6, inHigh, inLow, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "plus_di_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0};
        code = TA_PLUS_DI(0, 6, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "minus_di_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0};
        code = TA_MINUS_DI(0, 6, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "dx_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5};
        code = TA_DX(0, 7, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "adx_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0, 19.0, 20.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0, 16.0, 17.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5, 18.0, 19.0};
        code = TA_ADX(0, 9, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "adx_unstable_2") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0, 19.0, 20.0, 21.0, 22.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0, 16.0, 17.0, 18.0, 19.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5, 18.0, 19.0, 20.0, 21.0};
        TA_SetUnstablePeriod(TA_FUNC_UNST_ADX, 2);
        code = TA_ADX(0, 11, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "adxr_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0, 19.0, 20.0, 21.0, 22.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0, 16.0, 17.0, 18.0, 19.0};
        const double inClose[] = {9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5, 18.0, 19.0, 20.0, 21.0};
        code = TA_ADXR(0, 11, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ultosc_default") == 0) {
        const double inHigh[] = {10.0,11.0,15.0,13.0,14.0,16.0,18.0,17.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0,36.0,37.0,38.0,39.0,40.0};
        const double inLow[] = {8.0,9.0,10.0,11.0,12.0,14.0,15.0,14.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0,36.0,37.0};
        const double inClose[] = {9.0,10.0,12.0,12.5,13.0,15.0,16.0,15.5,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0,36.0,37.0,38.0,39.0};
        code = TA_ULTOSC(0, 29, inHigh, inLow, inClose, 7, 14, 28, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "stochf_default") == 0) {
        const double inHigh[] = {10.0,11.0,15.0,13.0,14.0,16.0,18.0,17.0,19.0,20.0,21.0,22.0};
        const double inLow[] = {8.0,9.0,10.0,11.0,12.0,14.0,15.0,14.0,16.0,17.0,18.0,19.0};
        const double inClose[] = {9.0,10.0,12.0,12.5,13.0,15.0,16.0,15.5,18.0,19.0,20.0,21.0};
        double outSecond[64] = {0.0};
        code = TA_STOCHF(0, 11, inHigh, inLow, inClose, 5, 3, TA_MAType_SMA, &outBegIdx, &outNbElement, outReal, outSecond);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outSecond[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "stoch_default") == 0) {
        const double inHigh[] = {10.0,11.0,15.0,13.0,14.0,16.0,18.0,17.0,19.0,20.0,21.0,22.0};
        const double inLow[] = {8.0,9.0,10.0,11.0,12.0,14.0,15.0,14.0,16.0,17.0,18.0,19.0};
        const double inClose[] = {9.0,10.0,12.0,12.5,13.0,15.0,16.0,15.5,18.0,19.0,20.0,21.0};
        double outSecond[64] = {0.0};
        code = TA_STOCH(0, 11, inHigh, inLow, inClose, 5, 3, TA_MAType_SMA, 3, TA_MAType_SMA, &outBegIdx, &outNbElement, outReal, outSecond);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outSecond[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "stochrsi_default") == 0) {
        const double inReal[] = {
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
            45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
            46.03, 46.41, 46.22, 45.64, 46.21, 46.25, 46.50, 46.70,
            46.90, 47.10, 47.30, 47.50, 47.70, 47.90
        };
        double outSecond[64] = {0.0};
        code = TA_STOCHRSI(0, 29, inReal, 14, 5, 3, TA_MAType_SMA, &outBegIdx, &outNbElement, outReal, outSecond);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outSecond[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "apo_sma_default") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0};
        code = TA_APO(0, 11, inReal, 3, 5, TA_MAType_SMA, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ppo_ema") == 0) {
        const double inReal[] = {10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0};
        code = TA_PPO(0, 11, inReal, 3, 5, TA_MAType_EMA, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "macd_basic") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0};
        code = TA_MACD(0, 17, inReal, 3, 6, 4, &outBegIdx, &outNbElement, outReal, outReal+32, outReal+48);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[32+i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[48+i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "macdfix_basic") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0};
        code = TA_MACDFIX(0, 29, inReal, 4, &outBegIdx, &outNbElement, outReal, outReal+32, outReal+48);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[32+i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[48+i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "dema_period_3") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0};
        code = TA_DEMA(0, 11, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "bbands_sma_default") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0};
        code = TA_BBANDS(0, 9, inReal, 5, 2.0, 2.0, TA_MAType_SMA, &outBegIdx, &outNbElement, outReal, outReal+64, outReal+128);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[64+i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[128+i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "bbands_ema_default") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0};
        code = TA_BBANDS(0, 9, inReal, 5, 2.0, 1.5, TA_MAType_EMA, &outBegIdx, &outNbElement, outReal, outReal+64, outReal+128);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[64+i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[128+i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "accbands_period_3") == 0) {
        const double inHigh[] = {10.0, 11.0, 12.0, 14.0, 13.0, 15.0};
        const double inLow[] = {8.0, 9.0, 10.0, 11.0, 11.5, 12.0};
        const double inClose[] = {9.0, 10.0, 11.0, 13.0, 12.0, 14.0};
        code = TA_ACCBANDS(0, 5, inHigh, inLow, inClose, 3, &outBegIdx, &outNbElement, outReal, outReal+64, outReal+128);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[64+i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[128+i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "mavp_sma") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0};
        const double inPeriods[] = {2.0,2.0,3.0,3.0,4.0,4.0,2.0,5.0,3.0,4.0};
        code = TA_MAVP(0, 9, inReal, inPeriods, 2, 5, TA_MAType_SMA, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "sar_default") == 0) {
        const double inHigh[] = {10.0,11.0,12.0,11.5,13.0,14.0,13.5,15.0};
        const double inLow[] = {9.0,9.5,10.5,10.0,11.0,12.5,12.0,13.0};
        code = TA_SAR(0, 7, inHigh, inLow, 0.02, 0.2, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "sarext_default") == 0) {
        const double inHigh[] = {10.0,11.0,12.0,11.5,13.0,14.0,13.5,15.0};
        const double inLow[] = {9.0,9.5,10.5,10.0,11.0,12.5,12.0,13.0};
        code = TA_SAREXT(0, 7, inHigh, inLow, 0.0, 0.0, 0.02, 0.02, 0.2, 0.02, 0.02, 0.2, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "kama_period_3") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,3.5,4.5,5.5,6.0,7.0,7.5,8.5,9.0};
        code = TA_KAMA(0, 11, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "kama_period_3_unstable_2") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,3.5,4.5,5.5,6.0,7.0,7.5,8.5,9.0,9.5,10.5};
        TA_SetUnstablePeriod(TA_FUNC_UNST_KAMA, 2);
        code = TA_KAMA(0, 13, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "mama_default") == 0) {
        const double inReal[] = {10.0,10.4,10.8,11.1,11.5,11.8,12.0,12.1,12.0,11.9,11.7,11.8,12.0,12.3,12.7,13.0,13.2,13.1,12.9,12.7,12.6,12.8,13.1,13.5,13.8,14.0,14.2,14.1,13.9,13.7,13.5,13.6,13.9,14.3,14.6,14.8,15.0,15.1,15.0,14.8,14.6,14.5,14.7,15.0,15.4};
        code = TA_MAMA(0, 44, inReal, 0.5, 0.05, &outBegIdx, &outNbElement, outReal, outReal+64);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[64+i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "mama_unstable_2") == 0) {
        const double inReal[] = {10.0,10.4,10.8,11.1,11.5,11.8,12.0,12.1,12.0,11.9,11.7,11.8,12.0,12.3,12.7,13.0,13.2,13.1,12.9,12.7,12.6,12.8,13.1,13.5,13.8,14.0,14.2,14.1,13.9,13.7,13.5,13.6,13.9,14.3,14.6,14.8,15.0,15.1,15.0,14.8,14.6,14.5,14.7,15.0,15.4,15.7,15.9};
        TA_SetUnstablePeriod(TA_FUNC_UNST_MAMA, 2);
        code = TA_MAMA(0, 46, inReal, 0.5, 0.05, &outBegIdx, &outNbElement, outReal, outReal+64);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[64+i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "ht_dcperiod_default") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0,36.0,37.0,38.0,39.0,40.0,41.0,42.0,43.0,44.0,45.0,46.0,47.0,48.0,49.0,50.0,51.0,52.0,53.0,54.0,55.0,56.0,57.0,58.0,59.0,60.0,61.0,62.0,63.0,64.0,65.0,66.0,67.0,68.0,69.0,70.0,71.0,72.0,73.0,74.0,75.0,76.0,77.0,78.0,79.0,80.0,81.0,82.0,83.0,84.0,85.0,86.0,87.0,88.0,89.0,90.0};
        code = TA_HT_DCPERIOD(0, 89, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ht_dcphase_default") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0,36.0,37.0,38.0,39.0,40.0,41.0,42.0,43.0,44.0,45.0,46.0,47.0,48.0,49.0,50.0,51.0,52.0,53.0,54.0,55.0,56.0,57.0,58.0,59.0,60.0,61.0,62.0,63.0,64.0,65.0,66.0,67.0,68.0,69.0,70.0,71.0,72.0,73.0,74.0,75.0,76.0,77.0,78.0,79.0,80.0,81.0,82.0,83.0,84.0,85.0,86.0,87.0,88.0,89.0,90.0};
        code = TA_HT_DCPHASE(0, 89, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ht_phasor_default") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0,36.0,37.0,38.0,39.0,40.0,41.0,42.0,43.0,44.0,45.0,46.0,47.0,48.0,49.0,50.0,51.0,52.0,53.0,54.0,55.0,56.0,57.0,58.0,59.0,60.0,61.0,62.0,63.0,64.0,65.0,66.0,67.0,68.0,69.0,70.0,71.0,72.0,73.0,74.0,75.0,76.0,77.0,78.0,79.0,80.0,81.0,82.0,83.0,84.0,85.0,86.0,87.0,88.0,89.0,90.0};
        code = TA_HT_PHASOR(0, 89, inReal, &outBegIdx, &outNbElement, outReal, outReal+64);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[64+i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "ht_sine_default") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0,36.0,37.0,38.0,39.0,40.0,41.0,42.0,43.0,44.0,45.0,46.0,47.0,48.0,49.0,50.0,51.0,52.0,53.0,54.0,55.0,56.0,57.0,58.0,59.0,60.0,61.0,62.0,63.0,64.0,65.0,66.0,67.0,68.0,69.0,70.0,71.0,72.0,73.0,74.0,75.0,76.0,77.0,78.0,79.0,80.0,81.0,82.0,83.0,84.0,85.0,86.0,87.0,88.0,89.0,90.0};
        code = TA_HT_SINE(0, 89, inReal, &outBegIdx, &outNbElement, outReal, outReal+64);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[64+i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "ht_trendline_default") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0,36.0,37.0,38.0,39.0,40.0,41.0,42.0,43.0,44.0,45.0,46.0,47.0,48.0,49.0,50.0,51.0,52.0,53.0,54.0,55.0,56.0,57.0,58.0,59.0,60.0,61.0,62.0,63.0,64.0,65.0,66.0,67.0,68.0,69.0,70.0,71.0,72.0,73.0,74.0,75.0,76.0,77.0,78.0,79.0,80.0,81.0,82.0,83.0,84.0,85.0,86.0,87.0,88.0,89.0,90.0};
        code = TA_HT_TRENDLINE(0, 89, inReal, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ht_trendmode_default") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0,36.0,37.0,38.0,39.0,40.0,41.0,42.0,43.0,44.0,45.0,46.0,47.0,48.0,49.0,50.0,51.0,52.0,53.0,54.0,55.0,56.0,57.0,58.0,59.0,60.0,61.0,62.0,63.0,64.0,65.0,66.0,67.0,68.0,69.0,70.0,71.0,72.0,73.0,74.0,75.0,76.0,77.0,78.0,79.0,80.0,81.0,82.0,83.0,84.0,85.0,86.0,87.0,88.0,89.0,90.0};
        code = TA_HT_TRENDMODE(0, 89, inReal, &outBegIdx, &outNbElement, (int*)outReal);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", ((int*)outReal)[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "tema_period_3") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0};
        code = TA_TEMA(0, 14, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "t3_period_5") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0};
        code = TA_T3(0, 34, inReal, 5, 0.7, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "t3_period_5_unstable_2") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0,28.0,29.0,30.0,31.0,32.0,33.0,34.0,35.0,36.0,37.0};
        TA_SetUnstablePeriod(TA_FUNC_UNST_T3, 2);
        code = TA_T3(0, 36, inReal, 5, 0.7, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "trima_period_5") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0};
        code = TA_TRIMA(0, 9, inReal, 5, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "trix_period_3") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0};
        code = TA_TRIX(0, 14, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ma_sma_period_3") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0};
        code = TA_MA(0, 5, inReal, 3, TA_MAType_SMA, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "ma_wma_period_3") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0};
        code = TA_MA(0, 5, inReal, 3, TA_MAType_WMA, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "macdext_basic") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,11.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0};
        code = TA_MACDEXT(0, 17, inReal, 3, TA_MAType_EMA, 6, TA_MAType_EMA, 4, TA_MAType_EMA, &outBegIdx, &outNbElement, outReal, outReal+32, outReal+48);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[32+i]);
        printf(" |");
        for (int i = 0; i < outNbElement; i++) printf(" %.17g", outReal[48+i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "linearreg_period_5") == 0) {
        const double inReal[] = {2.0,4.0,6.0,8.0,10.0,9.0,8.0,7.0,6.0,5.0};
        code = TA_LINEARREG(0, 9, inReal, 5, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "linearreg_angle_period_5") == 0) {
        const double inReal[] = {2.0,4.0,6.0,8.0,10.0,9.0,8.0,7.0,6.0,5.0};
        code = TA_LINEARREG_ANGLE(0, 9, inReal, 5, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "linearreg_intercept_period_5") == 0) {
        const double inReal[] = {2.0,4.0,6.0,8.0,10.0,9.0,8.0,7.0,6.0,5.0};
        code = TA_LINEARREG_INTERCEPT(0, 9, inReal, 5, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "linearreg_slope_period_5") == 0) {
        const double inReal[] = {2.0,4.0,6.0,8.0,10.0,9.0,8.0,7.0,6.0,5.0};
        code = TA_LINEARREG_SLOPE(0, 9, inReal, 5, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "tsf_period_5") == 0) {
        const double inReal[] = {2.0,4.0,6.0,8.0,10.0,9.0,8.0,7.0,6.0,5.0};
        code = TA_TSF(0, 9, inReal, 5, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "wma_period_3") == 0) {
        const double inReal[] = {1.0,2.0,3.0,4.0,5.0,6.0};
        code = TA_WMA(0, 5, inReal, 3, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "rsi_default") == 0) {
        const double inReal[] = {
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
            45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
            46.03, 46.41, 46.22, 45.64, 46.21
        };
        code = TA_RSI(0, 20, inReal, 14, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "rsi_metastock") == 0) {
        const double inReal[] = {
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
            45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
            46.03, 46.41, 46.22, 45.64, 46.21
        };
        TA_SetCompatibility(TA_COMPATIBILITY_METASTOCK);
        code = TA_RSI(0, 20, inReal, 14, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "rsi_unstable_2") == 0) {
        const double inReal[] = {
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
            45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
            46.03, 46.41, 46.22, 45.64, 46.21, 46.25, 46.50
        };
        TA_SetUnstablePeriod(TA_FUNC_UNST_RSI, 2);
        code = TA_RSI(0, 22, inReal, 14, &outBegIdx, &outNbElement, outReal);
    } else if (strcmp(argv[1], "cdldoji_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,12.5};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.5};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,12.05};
        int outInteger[64] = {0};
        code = TA_CDLDOJI(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdldoji_bodydoji_override") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,12.5};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.5};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,12.20};
        int outInteger[64] = {0};
        TA_SetCandleSettings(TA_BodyDoji, TA_RangeType_HighLow, 10, 0.4);
        code = TA_CDLDOJI(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdldragonflydoji_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,12.05};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,10.8};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,12.02};
        int outInteger[64] = {0};
        code = TA_CDLDRAGONFLYDOJI(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlgravestonedoji_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,13.2};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.99};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,12.02};
        int outInteger[64] = {0};
        code = TA_CDLGRAVESTONEDOJI(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlspinningtop_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,13.4};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.2};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,12.6};
        int outInteger[64] = {0};
        code = TA_CDLSPINNINGTOP(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlmarubozu_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,14.05};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.95};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,14.0};
        int outInteger[64] = {0};
        code = TA_CDLMARUBOZU(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdllongline_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,14.2};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,14.0};
        int outInteger[64] = {0};
        code = TA_CDLLONGLINE(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlshortline_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,12.8};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,12.6};
        int outInteger[64] = {0};
        code = TA_CDLSHORTLINE(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlhammer_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,11.0,10.4};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,11.6,10.65};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,10.2,9.4};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,11.4,10.6};
        int outInteger[64] = {0};
        code = TA_CDLHAMMER(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlengulfing_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,10.8};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.2,12.6};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,10.8,10.6};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,11.0,12.4};
        int outInteger[64] = {0};
        code = TA_CDLENGULFING(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlharami_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,10.8};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.2,11.3};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.8,10.7};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,11.2};
        int outInteger[64] = {0};
        code = TA_CDLHARAMI(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlharamicross_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,10.0,11.1};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.2,11.4};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.8,11.0};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,12.0,11.12};
        int outInteger[64] = {0};
        code = TA_CDLHARAMICROSS(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlbelthold_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,14.2};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.98};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,14.1};
        int outInteger[64] = {0};
        code = TA_CDLBELTHOLD(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlclosingmarubozu_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,14.12};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.6};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,14.1};
        int outInteger[64] = {0};
        code = TA_CDLCLOSINGMARUBOZU(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlhighwave_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,14.4};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,10.7};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,12.3};
        int outInteger[64] = {0};
        code = TA_CDLHIGHWAVE(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlinvertedhammer_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,11.0,9.6};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,11.2,10.8};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,10.6,9.55};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.8,9.8};
        int outInteger[64] = {0};
        code = TA_CDLINVERTEDHAMMER(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlshootingstar_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,13.3};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,14.5};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,13.25};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.1};
        int outInteger[64] = {0};
        code = TA_CDLSHOOTINGSTAR(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdltakuri_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,12.03};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,10.1};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,12.01};
        int outInteger[64] = {0};
        code = TA_CDLTAKURI(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlcounterattack_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,9.5};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,12.2};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.5,9.3};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,10.0};
        int outInteger[64] = {0};
        code = TA_CDLCOUNTERATTACK(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlhomingpigeon_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,11.6};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,11.7};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.7,11.0};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,11.1};
        int outInteger[64] = {0};
        code = TA_CDLHOMINGPIGEON(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlinneck_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,9.4};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,10.2};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.5,9.2};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,10.02};
        int outInteger[64] = {0};
        code = TA_CDLINNECK(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlonneck_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,9.3};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,9.9};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.5,9.1};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,9.5};
        int outInteger[64] = {0};
        code = TA_CDLONNECK(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlthrusting_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,9.3};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,10.7};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.5,9.1};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,10.6};
        int outInteger[64] = {0};
        code = TA_CDLTHRUSTING(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlmatchinglow_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,11.2};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,11.5};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.7,10.4};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,10.0};
        int outInteger[64] = {0};
        code = TA_CDLMATCHINGLOW(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdl2crows_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,13.6};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,14.2,13.8};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,12.1};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,14.0,12.8};
        int outInteger[64] = {0};
        code = TA_CDL2CROWS(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdl3blackcrows_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.6,11.9,11.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.0,12.0,11.1};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.7,10.9,10.0};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,11.8,10.8,10.0};
        int outInteger[64] = {0};
        code = TA_CDL3BLACKCROWS(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdldarkcloudcover_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,14.4};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,14.2,14.5};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,12.4};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,14.0,13.0};
        int outInteger[64] = {0};
        code = TA_CDLDARKCLOUDCOVER(0, 11, inOpen, inHigh, inLow, inClose, 0.5, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlpiercing_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,9.2};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,13.0};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.8,9.1};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,11.3};
        int outInteger[64] = {0};
        code = TA_CDLPIERCING(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlseparatinglines_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.2,14.1};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,10.0,11.98};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.2,14.0};
        int outInteger[64] = {0};
        code = TA_CDLSEPARATINGLINES(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlsticksandwich_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,10.1,11.1};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,11.3,11.2};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.9,10.05,9.9};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,11.2,10.0};
        int outInteger[64] = {0};
        code = TA_CDLSTICKSANDWICH(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdl3inside_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,10.8,10.7};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,11.3,10.9};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.8,10.7,9.8};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,11.2,9.9};
        int outInteger[64] = {0};
        code = TA_CDL3INSIDE(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdl3linestrike_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.3,12.8,14.2};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.1,13.6,14.1,14.3};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,12.2,12.7,11.7};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.5,14.0,11.8};
        int outInteger[64] = {0};
        code = TA_CDL3LINESTRIKE(0, 13, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdl3outside_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,9.6,12.8};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,12.9,13.2};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,10.0,9.4,12.6};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.2,12.7,13.0};
        int outInteger[64] = {0};
        code = TA_CDL3OUTSIDE(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlhangingman_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.3};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,12.55};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.1};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,12.45};
        int outInteger[64] = {0};
        code = TA_CDLHANGINGMAN(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdldojistar_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,14.2};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,14.1,14.35};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,14.15};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,14.0,14.22};
        int outInteger[64] = {0};
        code = TA_CDLDOJISTAR(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdleveningstar_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,14.2,13.9};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,14.1,14.35,13.95};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,14.15,12.6};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,14.0,14.22,12.8};
        int outInteger[64] = {0};
        code = TA_CDLEVENINGSTAR(0, 12, inOpen, inHigh, inLow, inClose, 0.3, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdleveningdojistar_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,14.2,13.9};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,14.1,14.28,13.95};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,14.16,12.6};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,14.0,14.21,12.8};
        int outInteger[64] = {0};
        code = TA_CDLEVENINGDOJISTAR(0, 12, inOpen, inHigh, inLow, inClose, 0.3, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlmorningstar_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,9.0,9.2};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,9.1,11.5};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.8,8.95,9.1};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,9.08,11.4};
        int outInteger[64] = {0};
        code = TA_CDLMORNINGSTAR(0, 12, inOpen, inHigh, inLow, inClose, 0.3, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlmorningdojistar_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,9.0,9.2};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,9.06,11.5};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.8,8.96,9.1};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,9.01,11.4};
        int outInteger[64] = {0};
        code = TA_CDLMORNINGDOJISTAR(0, 12, inOpen, inHigh, inLow, inClose, 0.3, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdllongleggeddoji_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,13.1};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,10.9};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,12.03};
        int outInteger[64] = {0};
        code = TA_CDLLONGLEGGEDDOJI(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlrickshawman_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,11.95};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.4,13.1};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,10.8};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,11.98};
        int outInteger[64] = {0};
        code = TA_CDLRICKSHAWMAN(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdltristar_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,14.2,14.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,14.3,14.05};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,14.15,13.95};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,12.02,14.22,13.98};
        int outInteger[64] = {0};
        code = TA_CDLTRISTAR(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlunique3river_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,10.9,9.4};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.1,11.0,9.9};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,9.5,9.3,9.35};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,10.0,9.6,9.8};
        int outInteger[64] = {0};
        code = TA_CDLUNIQUE3RIVER(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlbreakaway_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,14.4,15.1,15.9,14.6};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,14.2,15.0,15.7,16.4,14.7};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,14.3,14.9,15.5,13.1};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,14.0,14.9,15.5,16.1,13.4};
        int outInteger[64] = {0};
        code = TA_CDLBREAKAWAY(0, 14, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdltasukigap_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,13.2,13.8};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.1,14.2,13.9};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,13.1,12.4};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,14.0,12.6};
        int outInteger[64] = {0};
        code = TA_CDLTASUKIGAP(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlupsidegap2crows_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,14.4,14.9};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,14.2,15.0,15.1};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,14.3,13.3};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,14.0,14.6,13.8};
        int outInteger[64] = {0};
        code = TA_CDLUPSIDEGAP2CROWS(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlkicking_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,13.0,14.4};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.02,15.1};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,14.38};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,12.0,15.0};
        int outInteger[64] = {0};
        code = TA_CDLKICKING(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlkickingbylength_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,13.0,14.5};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.02,15.5};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,14.48};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,12.0,15.4};
        int outInteger[64] = {0};
        code = TA_CDLKICKINGBYLENGTH(0, 11, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlrisefall3methods_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,13.2,12.7,12.3,12.0,12.8};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,14.2,13.3,12.8,12.4,12.1,14.5};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,12.4,12.2,11.9,11.8,12.7};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,14.0,12.8,12.4,12.1,11.9,14.4};
        int outInteger[64] = {0};
        code = TA_CDLRISEFALL3METHODS(0, 15, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlmathold_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,13.4,14.2,13.9,13.6,14.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,14.2,14.3,14.25,13.95,14.05,15.2};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,13.3,13.8,13.5,13.2,13.95};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,14.0,14.0,13.95,13.7,13.4,15.1};
        int outInteger[64] = {0};
        code = TA_CDLMATHOLD(0, 15, inOpen, inHigh, inLow, inClose, 0.5, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlidentical3crows_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,11.02,10.04};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.05,11.05,10.05};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,10.9,10.0,9.0};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,11.0,10.0,9.0};
        int outInteger[64] = {0};
        code = TA_CDLIDENTICAL3CROWS(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdl3whitesoldiers_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.3,12.8};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.02,13.52,14.02};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,12.28,12.78};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.5,14.0};
        int outInteger[64] = {0};
        code = TA_CDL3WHITESOLDIERS(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdladvanceblock_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,13.0,13.4};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.02,13.9,14.3};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,12.95,13.35};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.6,13.8};
        int outInteger[64] = {0};
        code = TA_CDLADVANCEBLOCK(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlstalledpattern_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,12.4,13.15};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.3,14.02,13.62};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,12.35,13.1};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.2,14.0,13.55};
        int outInteger[64] = {0};
        code = TA_CDLSTALLEDPATTERN(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlgapsidesidewhite_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,13.2,13.19};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.1,14.1,14.0};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,13.15,13.14};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.9,13.85};
        int outInteger[64] = {0};
        code = TA_CDLGAPSIDESIDEWHITE(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdl3starsinsouth_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,11.3,10.95};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.05,11.35,10.9};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,10.3,10.6,10.7};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,11.0,10.8,10.75};
        int outInteger[64] = {0};
        code = TA_CDL3STARSINSOUTH(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlabandonedbaby_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,14.2,13.9};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,14.2,14.25,13.95};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,14.18,12.7};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,14.0,14.21,12.8};
        int outInteger[64] = {0};
        code = TA_CDLABANDONEDBABY(0, 12, inOpen, inHigh, inLow, inClose, 0.3, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlconcealbabyswall_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,11.0,10.2,10.25};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.02,11.02,10.7,10.9};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,10.9,10.1,9.8,9.6};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,11.0,10.2,9.9,9.7};
        int outInteger[64] = {0};
        code = TA_CDLCONCEALBABYSWALL(0, 13, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlhikkake_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6};
        const double inHigh[] = {11.4,11.6,12.0,11.7,11.5,11.2,11.1,11.05,11.0};
        const double inLow[] = {9.6,9.8,10.0,10.3,10.5,10.2,10.1,10.0,9.9};
        const double inClose[] = {11.0,11.2,11.5,11.1,10.8,10.4,11.3,10.2,10.1};
        int outInteger[64] = {0};
        code = TA_CDLHIKKAKE(0, 8, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlhikkakemod_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8};
        const double inHigh[] = {11.4,11.6,12.4,12.1,11.8,11.6,11.3,11.2,11.15,11.1};
        const double inLow[] = {9.6,9.8,10.0,10.4,10.6,10.8,10.2,10.1,10.0,9.9};
        const double inClose[] = {11.0,11.2,10.2,10.8,11.0,11.2,10.3,11.5,10.2,10.1};
        int outInteger[64] = {0};
        code = TA_CDLHIKKAKEMOD(0, 9, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlladderbottom_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,11.0,10.4,9.8,9.9};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,12.05,11.02,10.42,10.3,10.8};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,10.9,10.0,9.4,8.9,9.85};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,11.0,10.0,9.4,9.0,10.7};
        int outInteger[64] = {0};
        code = TA_CDLLADDERBOTTOM(0, 14, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    } else if (strcmp(argv[1], "cdlxsidegap3methods_basic") == 0) {
        const double inOpen[] = {10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.6,11.8,12.0,13.2,14.0};
        const double inHigh[] = {11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,13.2,13.1,14.1,14.05};
        const double inLow[] = {9.6,9.8,10.0,10.2,10.4,10.6,10.8,11.0,11.2,11.4,11.9,13.1,12.5};
        const double inClose[] = {11.0,11.2,11.4,11.6,11.8,12.0,12.2,12.4,12.6,12.8,13.0,14.0,12.6};
        int outInteger[64] = {0};
        code = TA_CDLXSIDEGAP3METHODS(0, 12, inOpen, inHigh, inLow, inClose, &outBegIdx, &outNbElement, outInteger);
        printf("%d %d %d", (int)code, outBegIdx, outNbElement);
        for (int i = 0; i < outNbElement; i++) printf(" %d", outInteger[i]);
        printf("\n");
        TA_Shutdown();
        return 0;
    #include "generated_oracle_cases.inc"
    } else {
        fprintf(stderr, "unknown case: %s\n", argv[1]);
        TA_Shutdown();
        return 4;
    }

    print_result(code, outBegIdx, outNbElement, outReal);
    TA_Shutdown();
    return 0;
}
