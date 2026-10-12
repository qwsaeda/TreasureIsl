/* ea:   0x0001ABD8 */
/* name: sub_1ABD8 */
/* size: 152 */
int __fastcall sub_1ABD8(_DWORD *a1)
{
  void (__fastcall ***v2)(_DWORD, int); // r3
  void *v3; // r4
  void **v4; // r4
  void **i; // r5

  sub_43CC4(a1);
  v2 = (void (__fastcall ***)(_DWORD, int))a1[255];
  if ( v2 != nullptr )
  {
    (**v2)(a1[255], 1);
    a1[255] = 0;
  }
  v3 = (void *)a1[284];
  if ( v3 != nullptr )
  {
    sub_45EFC(a1[284]);
    operator delete(v3);
    a1[284] = 0;
  }
  v4 = (void **)a1[258];
  for ( i = (void **)a1[259]; v4 != i; v4 += 9 )
  {
    operator delete(*v4);
    operator delete(v4[1]);
  }
  return sub_1AF68(a1 + 258, a1[258], a1[259]);
}

