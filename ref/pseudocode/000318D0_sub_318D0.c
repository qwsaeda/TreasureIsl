/* ea:   0x000318D0 */
/* name: sub_318D0 */
/* size: 144 */
int __fastcall sub_318D0(_DWORD *a1, unsigned __int8 *a2)
{
  int v4; // r6
  __int16 v6; // [sp+0h] [bp-18h] BYREF
  char v7; // [sp+4h] [bp-14h] BYREF

  v4 = sub_34470();
  if ( v4 != *a1 && *a2 >= (unsigned int)*(unsigned __int8 *)(v4 + 16) )
    return v4 + 17;
  v6 = *a2;
  return *(_DWORD *)((int (__fastcall *)(_DWORD *, char *, int, __int16 *))sub_35E8C)(a1, &v7, v4, &v6) + 17;
}

