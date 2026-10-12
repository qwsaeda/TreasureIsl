/* ea:   0x00048654 */
/* name: sub_48654 */
/* size: 904 */
void sub_48654(int a1, _DWORD *a2, ...)
{
  unsigned __int16 *v4; // r8
  __int16 v5; // r0
  __int16 v6; // r1
  unsigned __int16 v7; // r5
  int v8; // r3
  const void *v9; // r1
  int v10; // r9
  _DWORD *i; // r4
  __int16 v12; // r7
  int v13; // r5
  int v14; // r4
  int v15; // kr00_4
  int v16; // r6
  __int16 v17; // r5
  unsigned __int16 v18; // r3
  int v19; // r4
  __int16 v20; // r3
  int v21; // r0
  __int16 v22; // r5
  int v23; // r4
  int v24; // r0
  __int16 v25; // [sp+4h] [bp-54h] BYREF
  unsigned __int16 v26; // [sp+6h] [bp-52h] BYREF
  _WORD v27[6]; // [sp+8h] [bp-50h] BYREF
  void *v28[13]; // [sp+14h] [bp-44h] BYREF
  va_list varg_r2; // [sp+50h] [bp-8h] BYREF

  va_start(varg_r2, a2);
  if ( *(_DWORD *)(a1 + 20) == 0 )
    goto LABEL_27;
  v4 = (unsigned __int16 *)(a1 + 136);
  v5 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v6 = *(_WORD *)(*(_DWORD *)(a1 + 16) + 12);
  v25 = 0;
  v26 = 0;
  v7 = v5 + v6;
  sub_32134((int)v28);
  v8 = a2[7];
  if ( v8 != 0 )
  {
    if ( v8 != 1 )
      goto LABEL_7;
    v9 = (char *)a2 + 22;
  }
  else
  {
    v9 = a2 + 4;
  }
  memcpy(&v27[2], v9, 6u);
LABEL_7:
  v10 = 0;
  for ( i = (_DWORD *)*sub_1DDF0((int)varg_r2, v27); i != (_DWORD *)*sub_1DE08((void **)varg_r2, v27); i = sub_16A30(i) )
  {
    v10 = i[5];
    if ( *(unsigned __int8 *)(v10 + 50) == *(_DWORD *)(a1 + 8) )
      v7 += *(_WORD *)(v10 + 22);
  }
  v12 = 999;
  v13 = v7 / 10;
  v14 = 20 * HIBYTE(v27[3]) * v13 / 100;
  v15 = 15 * LOBYTE(v27[3]) * v13;
  v16 = *(_DWORD *)(a1 + 20);
  v17 = *(_WORD *)(v16 + 22);
  v18 = v14 + v15 / 0x64u + ((unsigned int)((unsigned __int64)(1374389535LL * v15) >> 32) >> 31);
  if ( v18 > 0x3E7u )
    v18 = 999;
  if ( v18 >= (unsigned int)*(unsigned __int16 *)(v16 + 22) )
  {
    sub_305F0((unsigned __int16 *)(a1 + 136), v18 - v17, (unsigned __int16 *)&v25, &v26, (int)v28);
    v19 = *(_DWORD *)(a1 + 20);
    v20 = *(_WORD *)(v19 + 22) + v25;
    if ( *(unsigned __int16 *)(v19 + 22) + (unsigned int)(unsigned __int16)v25 > 0x3E7 )
      v20 = 999;
    *(_WORD *)(v19 + 22) = v20;
    sub_31258(*(_DWORD *)(a1 + 20), v26);
    v21 = sub_51C14(a1, v28);
    sub_1654C((unsigned int *)(a1 + 144), v21);
  }
  else
  {
    sub_4A808(*(_DWORD *)(a1 + 20), (unsigned __int16)(v17 - v18), &v25, &v26);
    sub_16500((unsigned __int16 *)(a1 + 136), v25);
    sub_164D4((unsigned __int16 *)(a1 + 140), *v4);
    v22 = v26;
    sub_164D4((unsigned __int16 *)(a1 + 140), *v4);
    sub_16500((unsigned __int16 *)(a1 + 140), v22);
  }
  if ( *(_WORD *)(*(_DWORD *)(a1 + 20) + 22) == 0 && (unsigned int)sub_16C9C((unsigned __int16 *)(a1 + 136), 0) > 1 )
  {
    sub_305F0((unsigned __int16 *)(a1 + 136), 1u, (unsigned __int16 *)&v25, &v26, (int)v28);
    v23 = *(_DWORD *)(a1 + 20);
    if ( *(unsigned __int16 *)(v23 + 22) + (unsigned int)(unsigned __int16)v25 <= 0x3E7 )
      v12 = *(_WORD *)(v23 + 22) + v25;
    *(_WORD *)(v23 + 22) = v12;
    v24 = sub_51C14(a1, v28);
    sub_1654C((unsigned int *)(a1 + 144), v24);
  }
  if ( *(_WORD *)(*(_DWORD *)(a1 + 20) + 22) == 0 )
  {
    (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(*(_DWORD *)(a1 + 12) + 40) + 32))(
      *(_DWORD *)(*(_DWORD *)(a1 + 12) + 40),
      *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 8));
    sub_4AC18(*(_DWORD *)(a1 + 24) + 144);
    sub_4A89C(a1, *(unsigned __int8 *)(v10 + 48));
    *(_BYTE *)(*(_DWORD *)(a1 + 20) + 50) = 0;
  }
  sub_16ED8(v28);
  sub_32158(v28);
LABEL_27:
  sub_327B0((void **)varg_r2);
}

