/* ea:   0x00012434 */
/* name: sub_12434 */
/* size: 1180 */
int __fastcall sub_12434(int a1)
{
  int v2; // r0
  int v3; // r0
  void *v4; // r0
  int v5; // r0
  _DWORD *v6; // r4
  _DWORD *v7; // r0
  int v9; // r0
  int v10; // r6
  int v11; // r1
  int v12; // r0
  int v13; // r0
  int v14; // r0
  void *v15; // r0
  _DWORD *v16; // r4
  _DWORD *v17; // r0
  int v18; // r0
  int v19; // r0
  void *v20; // r0
  int v21; // r0
  _DWORD *v22; // r4
  _DWORD *v23; // r0
  _BYTE v24[8]; // [sp+Ch] [bp-DCh] BYREF
  _BYTE v25[16]; // [sp+14h] [bp-D4h] BYREF
  _BYTE v26[16]; // [sp+24h] [bp-C4h] BYREF
  _BYTE v27[16]; // [sp+34h] [bp-B4h] BYREF
  _BYTE v28[16]; // [sp+44h] [bp-A4h] BYREF
  wchar_t v29[74]; // [sp+54h] [bp-94h] BYREF

  sub_1333C(v25);
  sub_1333C(v26);
  if ( *(_DWORD *)(a1 + 1264) <= *(unsigned __int8 *)(a1 + 1050) + 1 )
  {
    v9 = *(_DWORD *)(a1 + 1132);
    v10 = 0;
    v11 = *(_DWORD *)(a1 + 1128);
    *(_DWORD *)(a1 + 1280) = 0;
    switch ( *(_DWORD *)(a1 + 4 * (v11 - v9) + 1168) )
    {
      case 1:
        v12 = 100;
        goto LABEL_15;
      case 2:
        v12 = 500;
        goto LABEL_15;
      case 3:
        v12 = 1000;
        goto LABEL_15;
      case 4:
        v12 = 800;
        goto LABEL_15;
      case 5:
      case 8:
        v12 = 5000;
        goto LABEL_15;
      case 6:
        v12 = 1200;
        goto LABEL_15;
      case 7:
        v12 = 2000;
        goto LABEL_15;
      case 9:
        v12 = 10000;
LABEL_15:
        *(_DWORD *)(a1 + 1280) = v12;
        break;
      default:
        break;
    }
    if ( *(_DWORD *)(a1 + 1024) >= *(_DWORD *)(a1 + 1280) && *(_WORD *)(a1 + 1048) != 0 )
    {
      *(_DWORD *)(a1 + 1268) = 30;
      sub_1309C(v25);
      sub_1309C(v26);
      v13 = sub_12F04(aDataUiPak);
      sub_134A4(v25, aDataUiPak, &aDataUiPak[v13], v24);
      v14 = sub_12F04(aDataMsgmMap);
      sub_134A4(v26, aDataMsgmMap, &aDataMsgmMap[v14], v24);
      v15 = operator new(0x610u);
      if ( v15 != nullptr )
        v10 = sub_462DC(v15, *(_DWORD *)(a1 + 4));
      *(_DWORD *)(a1 + 1272) = v10;
      sub_43D54(v10, a1);
      v16 = (_DWORD *)sub_132FC(v28, v26);
      v17 = (_DWORD *)sub_132FC(v27, v25);
      (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(**(_DWORD **)(a1 + 1272) + 4))(
        *(_DWORD *)(a1 + 1272),
        *v17,
        v17[1],
        v17[2],
        *v16,
        v16[1],
        v16[2]);
      memset(v29, 0, 0x80u);
      swprintf(v29, word_7D6F0, *(_DWORD *)(a1 + 1280), 1);
      sub_4661C(*(_DWORD *)(a1 + 1272), 2, 1, *(_DWORD *)(a1 + 1276), v29);
      sub_12F48(v26);
      sub_12EA0(v25);
      return 1;
    }
    else
    {
      *(_DWORD *)(a1 + 1268) = 20;
      sub_1309C(v25);
      sub_1309C(v26);
      v18 = sub_12F04(aDataUiPak);
      sub_134A4(v25, aDataUiPak, &aDataUiPak[v18], v24);
      v19 = sub_12F04(aDataMsgmMap);
      sub_134A4(v26, aDataMsgmMap, &aDataMsgmMap[v19], v24);
      v20 = operator new(0x610u);
      if ( v20 != nullptr )
        v21 = sub_462DC(v20, *(_DWORD *)(a1 + 4));
      else
        v21 = 0;
      *(_DWORD *)(a1 + 1272) = v21;
      ((void (*)(void))sub_43D54)();
      v22 = (_DWORD *)sub_132FC(v28, v26);
      v23 = (_DWORD *)sub_132FC(v27, v25);
      (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(**(_DWORD **)(a1 + 1272) + 4))(
        *(_DWORD *)(a1 + 1272),
        *v23,
        v23[1],
        v23[2],
        *v22,
        v22[1],
        v22[2]);
      memset(v29, 0, 0x80u);
      swprintf(v29, word_7D6D8);
      sub_4661C(*(_DWORD *)(a1 + 1272), 2, 0, *(_DWORD *)(a1 + 1276), v29);
      sub_132A0(v26);
      sub_132A0(v25);
      return 0;
    }
  }
  else
  {
    *(_DWORD *)(a1 + 1268) = 10;
    sub_1309C(v25);
    sub_1309C(v26);
    v2 = sub_12F04(aDataUiPak);
    sub_134A4(v25, aDataUiPak, &aDataUiPak[v2], v24);
    v3 = sub_12F04(aDataMsgmMap);
    sub_134A4(v26, aDataMsgmMap, &aDataMsgmMap[v3], v24);
    v4 = operator new(0x610u);
    if ( v4 != nullptr )
      v5 = sub_462DC(v4, *(_DWORD *)(a1 + 4));
    else
      v5 = 0;
    *(_DWORD *)(a1 + 1272) = v5;
    sub_43D54(v5, a1);
    v6 = (_DWORD *)sub_132FC(v27, v26);
    v7 = (_DWORD *)sub_132FC(v28, v25);
    (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(**(_DWORD **)(a1 + 1272) + 4))(
      *(_DWORD *)(a1 + 1272),
      *v7,
      v7[1],
      v7[2],
      *v6,
      v6[1],
      v6[2]);
    memset(v29, 0, 0x80u);
    swprintf(v29, word_7D714);
    sub_4661C(*(_DWORD *)(a1 + 1272), 2, 0, *(_DWORD *)(a1 + 1276), v29);
    sub_132A0(v26);
    sub_132A0(v25);
    return 0;
  }
}

