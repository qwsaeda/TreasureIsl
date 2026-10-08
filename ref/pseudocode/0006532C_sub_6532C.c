/* ea:   0x0006532C */
/* name: sub_6532C */
/* size: 132 */
int __fastcall sub_6532C(void **a1)
{
  _DWORD **i; // r4
  _DWORD *v3; // r0
  _DWORD *j; // r4

  for ( i = (_DWORD **)a1[5]; i != a1[6]; ++i )
  {
    v3 = *i;
    if ( *i != nullptr )
    {
      *v3 = off_7BD34;
      operator delete(v3);
    }
  }
  sub_65C98((int)(a1 + 5), a1[5], a1[6]);
  for ( j = a1[8]; j != a1[9]; ++j )
  {
    if ( *j != 0 )
      sub_653B0(*j, 1);
  }
  return sub_65DDC((int)(a1 + 8), a1[8], a1[9]);
}

