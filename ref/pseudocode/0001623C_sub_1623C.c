/* ea:   0x0001623C */
/* name: sub_1623C */
/* size: 160 */
int __fastcall sub_1623C(_DWORD *a1, unsigned __int16 *a2)
{
  int v4; // r7
  __int16 v5; // r0
  _BYTE v7[8]; // [sp+0h] [bp-28h] BYREF
  __int16 v8; // [sp+8h] [bp-20h] BYREF
  int v9; // [sp+Ch] [bp-1Ch]

  v4 = sub_16420();
  if ( v4 != *a1 && *a2 >= (unsigned int)*(unsigned __int16 *)(v4 + 16) )
    return v4 + 20;
  v5 = *a2;
  v9 = 0;
  v8 = v5;
  return *(_DWORD *)sub_1659C(a1, v7, v4, &v8) + 20;
}

