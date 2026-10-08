/* ea:   0x000453BC */
/* name: sub_453BC */
/* size: 84 */
BOOL __fastcall sub_453BC(int a1, unsigned __int8 a2)
{
  unsigned __int8 v3; // [sp+0h] [bp-8h] BYREF

  if ( a2 > 9u )
    return false;
  v3 = a2 - 1;
  return *(unsigned __int8 *)sub_318D0((_DWORD *)(a1 + 32), &v3) != 0;
}

