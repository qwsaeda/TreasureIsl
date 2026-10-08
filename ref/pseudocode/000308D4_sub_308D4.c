/* ea:   0x000308D4 */
/* name: sub_308D4 */
/* size: 164 */
int __fastcall sub_308D4(_DWORD *a1, char *a2)
{
  int v4; // r6
  char v5; // r0
  int v7; // [sp+0h] [bp-1Ch] BYREF
  _BYTE v8[2]; // [sp+4h] [bp-18h] BYREF
  __int16 v9; // [sp+6h] [bp-16h]
  __int16 v10; // [sp+8h] [bp-14h]

  v4 = sub_34358();
  if ( v4 != *a1 && (unsigned __int8)*a2 >= (unsigned int)*(unsigned __int8 *)(v4 + 16) )
    return v4 + 18;
  v5 = *a2;
  v7 = 0;
  v8[0] = v5;
  v9 = 0;
  v10 = 0;
  return *(_DWORD *)((int (__fastcall *)(_DWORD *, int *, int, _BYTE *))sub_35264)(a1, &v7, v4, v8) + 18;
}

