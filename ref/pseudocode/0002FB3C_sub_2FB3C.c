/* ea:   0x0002FB3C */
/* name: sub_2FB3C */
/* size: 148 */
int __fastcall sub_2FB3C(_DWORD *a1, int *a2)
{
  int v4; // r6
  int v5; // r0
  _BYTE v7[8]; // [sp+0h] [bp-20h] BYREF
  int v8; // [sp+8h] [bp-18h] BYREF
  __int16 v9; // [sp+Ch] [bp-14h]

  v4 = sub_34D9C();
  if ( v4 != *a1 && *a2 >= *(_DWORD *)(v4 + 16) )
    return v4 + 20;
  v5 = *a2;
  v9 = 0;
  v8 = v5;
  return *(_DWORD *)sub_37A34(a1, v7, v4, &v8) + 20;
}

