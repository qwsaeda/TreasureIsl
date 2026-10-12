/* ea:   0x00031DD8 */
/* name: sub_31DD8 */
/* size: 860 */
void sub_31DD8(int a1, _DWORD *a2, ...)
{
  unsigned __int16 *v4; // r7
  __int16 v5; // r0
  __int16 v6; // r1
  unsigned __int16 v7; // r5
  int v8; // r3
  const void *v9; // r1
  _DWORD *i; // r4
  int v11; // r2
  int v12; // r5
  int v13; // r4
  int v14; // r0
  int v15; // r5
  __int16 v16; // r6
  unsigned int v17; // r0
  unsigned __int16 v18; // r3
  unsigned __int16 v19; // r2
  unsigned __int16 v20; // r1
  int v21; // r4
  __int16 v22; // r3
  int v23; // r4
  __int16 v24; // r3
  int v25; // r0
  unsigned __int16 v26; // r4
  unsigned __int16 v27; // r3
  int v28; // r4
  int v29; // r0
  unsigned __int16 v30; // [sp+4h] [bp-54h] BYREF
  __int16 v31; // [sp+8h] [bp-50h] BYREF
  _WORD v32[8]; // [sp+Ch] [bp-4Ch] BYREF
  _DWORD v33[11]; // [sp+1Ch] [bp-3Ch] BYREF
  va_list varg_r2; // [sp+50h] [bp-8h] BYREF

  va_start(varg_r2, a2);
  v4 = (unsigned __int16 *)(a1 + 136);
  v5 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v6 = *(_WORD *)(*(_DWORD *)(a1 + 16) + 12);
  v31 = 0;
  v30 = 0;
  v7 = v5 + v6;
  sub_32134(v33);
  v8 = a2[7];
  if ( v8 == 0 )
  {
    v9 = a2 + 4;
    goto LABEL_5;
  }
  if ( v8 == 1 )
  {
    v9 = (char *)a2 + 22;
LABEL_5:
    memcpy(&v32[2], v9, 6u);
  }
  for ( i = (_DWORD *)*sub_1DDF0((int)varg_r2, v32); i != (_DWORD *)*sub_1DE08((void **)varg_r2, v32); i = sub_16A30(i) )
  {
    v11 = i[5];
    if ( *(unsigned __int8 *)(v11 + 50) == *(_DWORD *)(a1 + 8) )
      v7 += *(_WORD *)(v11 + 22);
  }
  v12 = v7 / 10;
  v13 = 20 * HIBYTE(v32[3]) * v12 / 100;
  v14 = LOBYTE(v32[3]) * v12;
  v15 = *(_DWORD *)(a1 + 16);
  v16 = 999;
  v17 = (int)((unsigned __int64)(20615843025LL * v14) >> 32) >> 5;
  v18 = v13 + v17 + (v17 >> 31);
  if ( v18 > 0x3E7u )
    v18 = 999;
  v19 = *(_WORD *)(v15 + 12);
  v20 = v18;
  if ( v18 < (unsigned int)v19 )
  {
    v26 = *(_WORD *)(v15 + 12);
    v27 = v19 - v18;
    if ( (unsigned __int16)(v19 - v20) <= (unsigned int)v26 )
      v26 = v27;
    v31 = v26;
    *(_WORD *)(v15 + 12) -= v26;
    sub_16500(v4, v31);
    sub_164D4(v4 + 2, *v4);
  }
  else
  {
    sub_305F0(v4, v18 - *(_WORD *)(v15 + 12), (unsigned __int16 *)&v31, &v30, (int)v33);
    v21 = *(_DWORD *)(a1 + 16);
    v22 = *(_WORD *)(v21 + 12) + v31;
    if ( *(unsigned __int16 *)(v21 + 12) + (unsigned int)(unsigned __int16)v31 > 0x3E7 )
      v22 = 999;
    *(_WORD *)(v21 + 12) = v22;
    v23 = *(_DWORD *)(a1 + 16);
    v24 = *(_WORD *)(v23 + 14) + v30;
    if ( *(unsigned __int16 *)(v23 + 14) + (unsigned int)v30 > 0x3E7 )
      v24 = 999;
    *(_WORD *)(v23 + 14) = v24;
    v25 = sub_51C14(a1, v33);
    sub_1654C((unsigned int *)(a1 + 144), v25);
  }
  if ( *(_WORD *)(*(_DWORD *)(a1 + 16) + 12) == 0 && (unsigned int)sub_16C9C(v4, 0) > 1 )
  {
    sub_305F0(v4, 1u, (unsigned __int16 *)&v31, &v30, (int)v33);
    v28 = *(_DWORD *)(a1 + 16);
    if ( *(unsigned __int16 *)(v28 + 12) + (unsigned int)(unsigned __int16)v31 <= 0x3E7 )
      v16 = *(_WORD *)(v28 + 12) + v31;
    *(_WORD *)(v28 + 12) = v16;
    v29 = sub_51C14(a1, v33);
    sub_1654C((unsigned int *)(a1 + 144), v29);
  }
  sub_16ED8(v33);
  sub_32158(v33);
  sub_303B8((void **)varg_r2);
  sub_30400((void **)varg_r2);
}

