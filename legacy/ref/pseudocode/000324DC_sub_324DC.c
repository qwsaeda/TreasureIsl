/* ea:   0x000324DC */
/* name: sub_324DC */
/* size: 724 */
void sub_324DC(int a1, int a2, ...)
{
  int v3; // r0
  unsigned __int16 v4; // r5
  unsigned __int16 v5; // r6
  _DWORD *i; // r4
  int v7; // r3
  unsigned __int16 v8; // r0
  int v9; // r5
  unsigned __int16 v10; // r4
  unsigned int v11; // r2
  int v12; // r0
  int v13; // r1
  unsigned __int16 v14; // r0
  _WORD *v15; // r3
  unsigned int v16; // r2
  unsigned __int16 v17; // r0
  unsigned __int16 v18; // r4
  unsigned __int16 v19; // r3
  _DWORD v20[10]; // [sp+0h] [bp-38h] BYREF
  va_list varg_r2; // [sp+30h] [bp-8h] BYREF

  va_start(varg_r2, a2);
  v3 = *(_DWORD *)(a1 + 20);
  if ( v3 != 0 && *(_WORD *)(v3 + 22) != 0 )
  {
    v4 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0) + *(_WORD *)(*(_DWORD *)(a1 + 16) + 12);
    v5 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1) + *(_WORD *)(*(_DWORD *)(a1 + 16) + 14);
    for ( i = (_DWORD *)*sub_1DDF0((int)varg_r2, v20); i != (_DWORD *)*sub_1DE08((void **)varg_r2, v20); i = sub_16A30(i) )
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
    v9 = *(_DWORD *)(a1 + 20);
    v10 = v8;
    if ( v8 > (unsigned int)*(unsigned __int16 *)(v9 + 22) )
      v10 = *(_WORD *)(v9 + 22);
    v11 = *(unsigned __int16 *)(v9 + 24);
    if ( v10 < v11 )
    {
      v17 = v11 - v10;
      v18 = *(_WORD *)(v9 + 24);
      v19 = v18;
      if ( v18 >= (unsigned int)v17 )
        v18 = v17;
      *(_WORD *)(v9 + 24) = v19 - v18;
      sub_32410((unsigned __int16 *)(a1 + 136), v18);
      sub_303B8((void **)varg_r2);
      sub_30400((void **)varg_r2);
    }
    else
    {
      v12 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
      v13 = *(unsigned __int16 *)(*(_DWORD *)(a1 + 20) + 24);
      if ( v10 <= (unsigned int)(v13 + v12) )
        v14 = v10 - v13;
      else
        v14 = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
      v15 = (_WORD *)(a1 + 140);
      v16 = *(unsigned __int16 *)(a1 + 140);
      if ( v14 < v16 )
        *v15 = v16 - v14;
      else
        *v15 = 0;
      sub_31258(*(_DWORD *)(a1 + 20), v14);
      sub_303B8((void **)varg_r2);
      sub_30400((void **)varg_r2);
    }
  }
  else
  {
    sub_327B0((void **)varg_r2);
  }
}

