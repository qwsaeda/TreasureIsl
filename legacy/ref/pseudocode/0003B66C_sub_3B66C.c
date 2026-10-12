/* ea:   0x0003B66C */
/* name: sub_3B66C */
/* size: 520 */
int __fastcall sub_3B66C(int result, __int16 *a2, char *a3, char *a4)
{
  int v7; // r3
  int v8; // r8
  int v9; // r6
  int v10; // r7
  int v11; // r5
  _WORD *v12; // r2
  int v13; // r0
  unsigned int v14; // r2
  int *v15; // r0
  int v16; // r6
  _WORD *v17; // r5
  _WORD *v18; // r0
  _BYTE *v19; // r0
  _WORD *v20; // r4
  int *v21; // [sp+8h] [bp-30h]
  int v22; // [sp+Ch] [bp-2Ch] BYREF
  unsigned int v23; // [sp+10h] [bp-28h] BYREF

  v21 = (int *)result;
  if ( a3 != a4 )
  {
    v7 = *(_DWORD *)(result + 4);
    v8 = (a4 - a3) >> 1;
    if ( v8 + 1 > (*(_DWORD *)(result + 8) - v7) >> 1 )
    {
      v13 = *(_DWORD *)result;
      v22 = (a4 - a3) >> 1;
      v14 = (v7 - v13) >> 1;
      v23 = v14;
      v15 = &v22;
      if ( v14 >= v8 )
        v15 = (int *)&v23;
      v16 = v14 + *v15 + 1;
      v17 = sub_1343C((int)(v21 + 2), v16);
      v18 = sub_13898(*v21, (int)a2, v17);
      v19 = memmove(v18, a3, a4 - a3);
      v20 = sub_186B8(a2, v21[1], &a4[v19 - a3]);
      *v20 = 0;
      if ( *v21 != 0 )
        free((void *)*v21);
      *v21 = (int)v17;
      result = (int)&v17[v16];
      v21[1] = (int)v20;
      v21[2] = result;
    }
    else
    {
      v9 = (v7 - (int)a2) >> 1;
      v10 = *(_DWORD *)(result + 4);
      if ( v9 < v8 )
      {
        v11 = (int)&a3[2 * v9 + 2];
        if ( a4 != (char *)v11 )
          memmove((void *)(v7 + 2), &a3[2 * v9 + 2], (size_t)&a4[-v11]);
        v12 = (_WORD *)(v21[1] + 2 * (v8 - v9));
        v21[1] = (int)v12;
        sub_13898((int)a2, v10 + 2, v12);
        v21[1] += 2 * v9;
        result = (2 * v9 + 2) >> 1;
        if ( result != 0 )
          return (int)memcpy(a2, a3, 2 * result);
      }
      else
      {
        sub_3B630(v10 - 2 * v8 + 2, v7 + 2, (_WORD *)(v10 + 2));
        v21[1] += 2 * v8;
        if ( v9 - v8 != -1 )
          memmove(&a2[v8], a2, 2 * (v9 - v8 + 1));
        result = v8;
        if ( v8 != 0 )
          return (int)memcpy(a2, a3, 2 * v8);
      }
    }
  }
  return result;
}

