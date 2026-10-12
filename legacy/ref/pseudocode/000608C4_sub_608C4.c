/* ea:   0x000608C4 */
/* name: sub_608C4 */
/* size: 252 */
void __fastcall sub_608C4(int a1, _DWORD *a2, _DWORD *a3, int a4)
{
  int v7; // r5
  _BYTE v8[8]; // [sp+8h] [bp-68h] BYREF
  void *v9[4]; // [sp+10h] [bp-60h] BYREF
  int v10; // [sp+20h] [bp-50h]
  void *v11[3]; // [sp+24h] [bp-4Ch] BYREF
  int v12; // [sp+30h] [bp-40h] BYREF
  void *v13[3]; // [sp+34h] [bp-3Ch] BYREF
  int v14; // [sp+40h] [bp-30h] BYREF
  _DWORD v15[3]; // [sp+44h] [bp-2Ch] BYREF
  int v16; // [sp+50h] [bp-20h] BYREF

  if ( a2 == nullptr )
    __debugbreak();
  v12 = sub_19A4C((_WORD *)*a3);
  sub_132FC(v13, a3);
  v7 = sub_61890(a2, &v12);
  sub_12F48(v13);
  if ( v7 == *a2 )
  {
    sub_132FC(v9, (_DWORD *)a4);
    v10 = sub_19A4C((_WORD *)*a3);
    sub_132FC(v11, a3);
    v14 = v10;
    sub_132FC(v15, v11);
    sub_60614(&v16, v9);
    sub_60C54(a2, v8, &v14);
    sub_19A98((int)&v14);
    sub_12F48(v11);
    sub_12F48(v9);
  }
  else if ( a4 != v7 + 32 )
  {
    sub_17890(v7 + 32, *(__int16 **)a4, *(__int16 **)(a4 + 4));
  }
}

