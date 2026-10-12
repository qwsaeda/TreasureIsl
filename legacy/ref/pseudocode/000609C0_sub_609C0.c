/* ea:   0x000609C0 */
/* name: sub_609C0 */
/* size: 196 */
int __fastcall sub_609C0(_DWORD *a1, int a2, unsigned int a3, int a4)
{
  int v6; // r5
  int v7; // r2
  int *v8; // r0
  int v9; // r1
  int v10; // r0
  unsigned __int8 v12; // [sp+8h] [bp-38h]
  int v13; // [sp+Ch] [bp-34h] BYREF
  int v14; // [sp+10h] [bp-30h] BYREF
  char v15[28]; // [sp+14h] [bp-2Ch] BYREF
  _DWORD *varg_r0; // [sp+30h] [bp-10h]
  int varg_r1; // [sp+34h] [bp-Ch]
  _DWORD varg_r2[2]; // [sp+38h] [bp-8h] BYREF

  varg_r0 = a1;
  varg_r1 = a2;
  varg_r2[0] = a3;
  varg_r2[1] = a4;
  v6 = *a1;
  v7 = (a1[1] - *a1) >> 1;
  if ( v7 == 0 )
    return -1;
  v13 = v7 - 1;
  v8 = varg_r2;
  if ( a3 >= v7 - 1 )
    v8 = &v13;
  v9 = *(_DWORD *)sub_615D4(v15, v6 + 2 * (*v8 + 1), v6, a2, a2 + 2 * a4, v12);
  v10 = *a1;
  v13 = v9;
  v14 = v10;
  if ( sub_13470(&v13, &v14) )
    return (v13 - *a1 - 2) >> 1;
  else
    return -1;
}

