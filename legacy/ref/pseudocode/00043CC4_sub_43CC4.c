/* ea:   0x00043CC4 */
/* name: sub_43CC4 */
/* size: 144 */
int __fastcall sub_43CC4(_DWORD *a1)
{
  int result; // r0
  void *v3; // r4
  void *v4; // r4
  void *v5; // r4

  result = a1[254];
  if ( result == 0 )
  {
    v3 = (void *)a1[247];
    if ( v3 != nullptr )
    {
      sub_45EFC(a1[247]);
      operator delete(v3);
      a1[247] = 0;
    }
    sub_44AAC(a1 + 244);
    v4 = (void *)a1[250];
    if ( v4 != nullptr )
    {
      sub_45EFC(a1[250]);
      operator delete(v4);
      a1[250] = 0;
    }
    v5 = (void *)a1[251];
    if ( v5 != nullptr )
    {
      sub_45EFC(a1[251]);
      operator delete(v5);
      a1[251] = 0;
    }
    a1[123] = 0;
    a1[254] = 1;
    return 1;
  }
  return result;
}

