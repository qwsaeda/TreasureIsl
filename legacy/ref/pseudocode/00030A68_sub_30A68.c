/* ea:   0x00030A68 */
/* name: sub_30A68 */
/* size: 20 */
void __fastcall sub_30A68(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

