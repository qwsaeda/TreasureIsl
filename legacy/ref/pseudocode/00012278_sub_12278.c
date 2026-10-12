/* ea:   0x00012278 */
/* name: sub_12278 */
/* size: 372 */
int __fastcall sub_12278(_DWORD *a1)
{
  int v2; // r0
  int v3; // r0
  void *v4; // r0
  int v5; // r0
  _DWORD *v6; // r4
  _DWORD *v7; // r0
  _BYTE v9[8]; // [sp+Ch] [bp-D8h] BYREF
  _BYTE v10[16]; // [sp+14h] [bp-D0h] BYREF
  _BYTE v11[16]; // [sp+24h] [bp-C0h] BYREF
  _BYTE v12[16]; // [sp+34h] [bp-B0h] BYREF
  _DWORD v13[4]; // [sp+44h] [bp-A0h] BYREF
  wchar_t v14[72]; // [sp+54h] [bp-90h] BYREF

  sub_1333C(v10);
  sub_1333C(v11);
  if ( *(_DWORD *)(a1[259] + 4 * a1[315] - 4) != 0 )
  {
    a1[317] = 40;
    sub_1309C(v10);
    sub_1309C(v11);
    v2 = sub_12F04(aDataUiPak);
    sub_134A4(v10, aDataUiPak, &aDataUiPak[v2], v9);
    v3 = sub_12F04(aDataMsgmMap);
    sub_134A4(v11, aDataMsgmMap, &aDataMsgmMap[v3], v9);
    v4 = operator new(0x610u);
    if ( v4 != nullptr )
      v5 = sub_462DC(v4, a1[1]);
    else
      v5 = 0;
    a1[318] = v5;
    sub_43D54();
    v6 = (_DWORD *)sub_132FC(v12, v11);
    v7 = (_DWORD *)sub_132FC(v13, v10);
    (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)a1[318] + 4))(
      a1[318],
      *v7,
      v7[1],
      v7[2],
      *v6,
      v6[1],
      v6[2]);
    memset(v14, 0, 0x80u);
    swprintf(v14, word_7D694);
    sub_4661C(a1[318], 2, 1, a1[319], v14);
  }
  sub_132A0(v11);
  sub_12F48(v10);
  return 1;
}

