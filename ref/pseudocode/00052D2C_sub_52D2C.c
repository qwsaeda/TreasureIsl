/* ea:   0x00052D2C */
/* name: sub_52D2C */
/* size: 80 */
int __fastcall sub_52D2C(_DWORD *a1, _WORD *a2)
{
  bool v4; // zf
  int result; // r0

  if ( sub_12F04(a2) != (a1[1] - *a1) >> 1 || (v4 = sub_52DB4(*a1, a2) == 0, LOBYTE(result) = 1, !v4) )
    LOBYTE(result) = 0;
  return (unsigned __int8)result;
}

