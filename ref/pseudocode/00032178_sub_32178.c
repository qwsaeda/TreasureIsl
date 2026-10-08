/* ea:   0x00032178 */
/* name: sub_32178 */
/* size: 664 */
void sub_32178(int a1, int a2, ...)
{
  unsigned __int16 *v3; // r8
  unsigned __int16 v4; // r5
  unsigned __int16 v5; // r6
  _DWORD *i; // r4
  int v7; // r3
  unsigned __int16 v8; // r0
  int v9; // r5
  unsigned __int16 v10; // r2
  __int16 v11; // r6
  unsigned int v12; // r3
  unsigned int v13; // r4
  int v14; // r0
  int v15; // r1
  unsigned __int16 v16; // r0
  _WORD *v17; // r4
  unsigned int v18; // r3
  int v19; // r3
  unsigned __int16 v20; // r0
  unsigned int v21; // r4
  __int16 v22; // r3
  _DWORD v23[10]; // [sp+0h] [bp-38h] BYREF
  va_list varg_r2; // [sp+30h] [bp-8h] BYREF

  va_start(varg_r2, a2);
  v3 = (unsigned __int16 *)(a1 + 136);
  v4 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0) + *(_WORD *)(*(_DWORD *)(a1 + 16) + 12);
  v5 = sub_16C9C(v3, 1) + *(_WORD *)(*(_DWORD *)(a1 + 16) + 14);
  for ( i = (_DWORD *)*sub_1DDF0((int)varg_r2, v23); i != (_DWORD *)*sub_1DE08((void **)varg_r2, v23); i = sub_16A30(i) )
  {
    v7 = i[5];
    if ( *(unsigned __int8 *)(v7 + 50) == *(_DWORD *)(a1 + 8) )
    {
      v4 += *(_WORD *)(v7 + 22);
      v5 += *(_WORD *)(v7 + 24);
    }
  }
  if ( v4 != 0 )
    v8 = *(unsigned __int16 *)(*(_DWORD *)(a1 + 16) + 12) * v5 / v4;
  else
    v8 = 0;
  v9 = *(_DWORD *)(a1 + 16);
  v10 = v8;
  v11 = 999;
  if ( v8 > 0x3E7u )
    v10 = 999;
  v12 = *(unsigned __int16 *)(v9 + 14);
  v13 = v10;
  if ( v10 < v12 )
  {
    v20 = v12 - v10;
    v21 = *(unsigned __int16 *)(v9 + 14);
    v22 = v21;
    if ( v21 >= v20 )
      v21 = v20;
    *(_WORD *)(v9 + 14) = v22 - v21;
    sub_32410(v3, v21);
    sub_303B8((void **)varg_r2);
    sub_30400((void **)varg_r2);
  }
  else
  {
    v14 = sub_16C9C(v3, 1);
    v15 = *(unsigned __int16 *)(*(_DWORD *)(a1 + 16) + 14);
    if ( v13 <= v15 + v14 )
      v16 = v13 - v15;
    else
      v16 = sub_16C9C(v3, 1);
    v17 = (_WORD *)(a1 + 140);
    v18 = *(unsigned __int16 *)(a1 + 140);
    if ( v16 < v18 )
      *v17 = v18 - v16;
    else
      *v17 = 0;
    v19 = *(_DWORD *)(a1 + 16);
    if ( v16 + (unsigned int)*(unsigned __int16 *)(v19 + 14) <= 0x3E7 )
      v11 = v16 + *(_WORD *)(v19 + 14);
    *(_WORD *)(v19 + 14) = v11;
    sub_303B8((void **)varg_r2);
    sub_30400((void **)varg_r2);
  }
}

